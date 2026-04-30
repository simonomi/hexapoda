#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::enum_glob_use)]
#![feature(get_disjoint_mut_helpers)]
#![feature(exact_bitshifts)]
#![feature(hash_set_entry)]
#![feature(trim_prefix_suffix)]
#![feature(const_trait_impl)]

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

// TODO:
// - click and drag selection
// - `go` goto entered offset
// - search
//   - `/` hex, `A-/` ascii
//   - if non-hex-digit typed, search ascii
// - inspector translations for varint
// - M mark at selected offset? (like Jm)
// - diffing
//   - doesn't have to be anything fancy, just compare each byte 1:1
//   - sync scroll ? sync selections ??
// - s/A-k/A-K
//   - sm select marks
// - C-a/C-x
// - +/- to edit selected bytes by amount ?
//   - operate on entire selection (u16/u32/etc)
//   - hex or decimal ?
// - modifications
//   - insert/append
//     - mode
//     - add to edit history when *leaving* insert mode
//   - replace-and-keep-going
//     - mode
//   - change
// - edit character panel
//   - modifier on existing keys like teehee? or jump to panel?
//     - A-r replaces with ASCII ?
//     - if jump to panel, space?
// - jumplist
// - p
// - [/] to cycle view offset?

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
	
	while !app.should_quit {
		terminal.draw(|frame| {
			frame.render_widget(&app, frame.area());
		}).unwrap();
		
		app.handle_events(&mut terminal);
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
