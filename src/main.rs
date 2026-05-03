#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::enum_glob_use)]

use arguments::Arguments;
use clap::Parser;
use app::App;
use crossterm::{QueueableCommand, event::{DisableMouseCapture, EnableMouseCapture}};
use crate::config::Config;

mod app;
mod buffer;
mod popup;
mod config;
mod cursor;
mod action;
mod edit_action;
mod arguments;
mod window_size;
mod utilities;

const BYTES_PER_LINE: usize = 0x10;
const BYTES_PER_CHUNK: usize = 4;
const CHUNKS_PER_LINE: usize = BYTES_PER_LINE / BYTES_PER_CHUNK;

const LINES_OF_PADDING: usize = 5;
const BYTES_OF_PADDING: usize = LINES_OF_PADDING * BYTES_PER_LINE;

fn main() {
	let arguments = Arguments::parse();
	
	if arguments.show_config_path {
		if let Some(path) = Config::path(arguments.config) {
			println!("{}", path.display());
		} else {
			#[cfg(unix)] {
				println!("currently, no config file will be used. define the environment variable XDG_CONFIG_HOME or use the -c/--config option to provide one");
			}
			#[cfg(windows)] {
				println!("currently, no config file will be used. use the -c/--config option to provide one");
			}
		}
		return;
	}
	
	let mut app = App::new(
		arguments.config,
		&arguments.files
	);
	
	let mut terminal = ratatui::init();
	crossterm::terminal::enable_raw_mode().unwrap();
	terminal.backend_mut().queue(EnableMouseCapture).unwrap();
	
	let mut should_redraw = true;
	
	while !app.should_quit {
		if should_redraw {
			terminal.draw(|frame| {
				frame.render_widget(&app, frame.area());
			}).unwrap();
		}
		
		should_redraw = app.handle_events(&mut terminal);
	}
	
	terminal.backend_mut().queue(DisableMouseCapture).unwrap();
	crossterm::terminal::disable_raw_mode().unwrap();
	ratatui::restore();
	
	// dbg!(app.edit_history);
	
	// dbg!(app.primary_cursor_register);
	// dbg!(app.other_cursor_registers);
	
	for log in app.logs {
		println!("{log}");
	}
	
	for log in app.buffers.iter().flat_map(|buffer| &buffer.logs) {
		println!("{log}");
	}
}
