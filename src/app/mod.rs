use std::{cmp::min, env, fs::File, io::Read, process::exit};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::BYTES_PER_LINE;

mod widget;

#[derive(Debug)]
pub struct App {
	pub contents: Vec<u8>,
	pub scroll_position: usize,
	// pub cursor_position: usize,
	pub should_quit: bool
}

impl App {
	pub fn init() -> Self {
		let input_files: Vec<_> = env::args().skip(1).collect();
		
		if input_files.is_empty() {
			println!("please provide at least one file as input");
			exit(1);
		}
		
		assert!(input_files.len() == 1);
		
		let file_name = input_files.first().unwrap();
		
		let file = File::open(file_name);
		let mut contents = Vec::new();
		file.unwrap().read_to_end(&mut contents).unwrap();
		
		Self {
			contents,
			scroll_position: 0,
			// cursor_position: 0,
			should_quit: false,
		}
	}
	
	pub fn handle_events(&mut self) {
		match event::read().unwrap() {
			Event::Key(key_event) if key_event.code == KeyCode::Char('q') => {
				self.should_quit = true;
			}
			Event::Key(key_event) if key_event.code == KeyCode::Char('e') &&
			                         key_event.modifiers.contains(KeyModifiers::CONTROL) => {
				let max_scroll_position = self.contents.len() - 0x50;
				self.scroll_position = min(self.scroll_position + BYTES_PER_LINE, max_scroll_position);
			}
			Event::Key(key_event) if key_event.code == KeyCode::Char('y') &&
			                         key_event.modifiers.contains(KeyModifiers::CONTROL) => {
				self.scroll_position = self.scroll_position.saturating_sub(BYTES_PER_LINE);
			}
			_ => {}
		}
	}
}
