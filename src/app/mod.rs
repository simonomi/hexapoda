use std::{cmp::min, env, fs::File, io::Read, path::PathBuf, process::exit};
use crossterm::{event::{self, Event, KeyCode, KeyModifiers}, terminal::window_size};
use ratatui::style::Color;

use crate::{BYTES_PER_LINE, cursor::Cursor};

mod widget;

#[derive(Debug)]
pub struct App {
	pub file_name: String,
	pub contents: Vec<u8>,
	pub window_rows: usize,
	pub scroll_position: usize,
	pub cursor: Cursor,
	pub should_quit: bool,
	pub mode: Mode,
	pub partial_action: Option<PartialAction>,
	pub logs: Vec<String>,
}

#[derive(Debug)]
pub enum Mode {
	Normal, Select, Insert
}

#[derive(Debug)]
pub enum PartialAction {
	Goto, Zview, Replace
}

impl Mode {
	pub const fn label(&self) -> &'static str {
		match self {
			Self::Normal => " NORMAL ",
			Self::Select => " SELECT ",
			Self::Insert => " INSERT ",
		}
	}
	
	pub const fn color(&self) -> Color {
		match self {
			Self::Normal => Color::Blue,
			Self::Select => Color::Yellow,
			Self::Insert => Color::Green,
		}
	}
}

impl PartialAction {
	pub const fn label(&self) -> &'static str {
		match self {
			Self::Goto => "g",
			Self::Zview => "z",
			Self::Replace => "r",
		}
	}
}

impl App {
	pub fn init() -> Self {
		let input_files: Vec<_> = env::args().skip(1).collect();
		
		if input_files.is_empty() {
			println!("please provide at least one file as input");
			exit(1);
		}
		
		assert!(input_files.len() == 1);
		
		let file_path: PathBuf = input_files.first().unwrap().into();
		
		let file = File::open(&file_path);
		let mut contents = Vec::new();
		file.unwrap().read_to_end(&mut contents).unwrap();
		
		Self {
			file_name: file_path.file_name().unwrap().to_str().unwrap().to_owned(),
			contents,
			// -1 because of the status line
			window_rows: window_size().unwrap().rows as usize - 1,
			scroll_position: 0,
			cursor: Cursor::default(),
			should_quit: false,
			mode: Mode::Normal,
			partial_action: None,
			logs: Vec::new(),
		}
	}
	
	// in bytes
	const fn screen_size(&self) -> usize {
		self.window_rows * BYTES_PER_LINE
	}
	
	#[allow(clippy::too_many_lines)]
	pub fn handle_events(&mut self) {
		#[allow(clippy::collapsible_match)]
		match (&self.mode, event::read().unwrap(), &self.partial_action) {
			(Mode::Normal, Event::Resize(_, height), _) => {
				// -1 because of the status line
				self.window_rows = height as usize - 1;
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			if key_event.code == KeyCode::Char('q') => {
				self.should_quit = true;
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			if key_event.modifiers.contains(KeyModifiers::CONTROL) &&
			   key_event.code == KeyCode::Char('e') => {
				self.scroll_position = min(
					self.scroll_position + BYTES_PER_LINE,
					self.contents.len() - (5 * BYTES_PER_LINE)
				);
				self.cursor.clamp(self.scroll_position, self.screen_size());
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			if key_event.modifiers.contains(KeyModifiers::CONTROL) &&
			   key_event.code == KeyCode::Char('y') => {
				self.scroll_position = self.scroll_position.saturating_sub(BYTES_PER_LINE);
				self.cursor.clamp(self.scroll_position, self.screen_size());
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			if key_event.modifiers.contains(KeyModifiers::CONTROL) &&
			   key_event.code == KeyCode::Char('d') => {
				let head_offset = self.cursor.head - self.scroll_position;
				let tail_offset = self.cursor.tail - self.scroll_position;
				
				self.scroll_position = min(
					self.scroll_position + (self.screen_size() / 2).next_multiple_of(BYTES_PER_LINE),
					self.contents.len() - (5 * BYTES_PER_LINE)
				);
				
				self.cursor.head = (self.scroll_position + head_offset).min(self.contents.len() - 1);
				self.cursor.tail = (self.scroll_position + tail_offset).min(self.contents.len() - 1);
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			if key_event.modifiers.contains(KeyModifiers::CONTROL) &&
			   key_event.code == KeyCode::Char('u') => {
				let head_offset = self.cursor.head - self.scroll_position;
				let tail_offset = self.cursor.tail - self.scroll_position;
				
				self.scroll_position = self.scroll_position.saturating_sub(
					(self.screen_size() / 2).next_multiple_of(BYTES_PER_LINE)
				);
				
				self.cursor.head = (self.scroll_position + head_offset).min(self.contents.len() - 1);
				self.cursor.tail = (self.scroll_position + tail_offset).min(self.contents.len() - 1);
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			if key_event.modifiers.contains(KeyModifiers::CONTROL) &&
			   key_event.code == KeyCode::Char('f') => {
				self.scroll_position = min(
					self.scroll_position + self.screen_size(),
					self.contents.len() - (5 * BYTES_PER_LINE)
				);
				self.cursor.clamp(self.scroll_position, self.screen_size());
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			if key_event.modifiers.contains(KeyModifiers::CONTROL) &&
			   key_event.code == KeyCode::Char('b') => {
				self.scroll_position = self.scroll_position.saturating_sub(
					self.screen_size()
				);
				self.cursor.clamp(self.scroll_position, self.screen_size());
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char('g') => {
				self.partial_action = Some(PartialAction::Goto);
			}
			
			(Mode::Normal, Event::Key(key_event), Some(PartialAction::Goto))
			   if key_event.code == KeyCode::Char('g') => {
				self.partial_action = None;
				self.cursor.head %= BYTES_PER_LINE;
				self.cursor.collapse();
				self.clamp_screen_to_cursor();
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char('G') => {
				self.cursor.head = previous_multiple_of(BYTES_PER_LINE, self.contents.len()) +
					(self.cursor.head % BYTES_PER_LINE);
				
				self.cursor.collapse();
				self.clamp_screen_to_cursor();
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char('z') => {
				self.partial_action = Some(PartialAction::Zview);
			}
			
			(Mode::Normal, Event::Key(key_event), Some(PartialAction::Zview))
			   if key_event.code == KeyCode::Char('z') => {
				self.partial_action = None;
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char('i') ||
			   key_event.code == KeyCode::Up => {
				self.partial_action = None;
				if self.cursor.head >= BYTES_PER_LINE {
					self.cursor.head -= BYTES_PER_LINE;
					self.cursor.collapse();
				
					self.clamp_screen_to_cursor();
				}
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char('j') ||
			   key_event.code == KeyCode::Left => {
				if self.cursor.head >= 1 {
					self.cursor.head -= 1;
					self.cursor.collapse();
					
					self.clamp_screen_to_cursor();
				}
			}
			
			(Mode::Normal, Event::Key(key_event), Some(PartialAction::Goto))
			   if key_event.code == KeyCode::Char('j') ||
			   key_event.code == KeyCode::Left => {
				self.partial_action = None;
				self.cursor.head -= self.cursor.head % BYTES_PER_LINE;
				self.cursor.collapse();
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char('k') ||
			   key_event.code == KeyCode::Down => {
				if self.contents.len() - 1 - self.cursor.head >= BYTES_PER_LINE {
					self.cursor.head += BYTES_PER_LINE;
					self.cursor.collapse();
					
					self.clamp_screen_to_cursor();
				}
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char('l') ||
			   key_event.code == KeyCode::Right => {
				if self.contents.len() - 1 - self.cursor.head >= 1 {
					self.cursor.head += 1;
					self.cursor.collapse();
					
					self.clamp_screen_to_cursor();
				}
			}
			
			(Mode::Normal, Event::Key(key_event), Some(PartialAction::Goto))
			   if key_event.code == KeyCode::Char('l') ||
			   key_event.code == KeyCode::Right => {
				self.partial_action = None;
				self.cursor.head += BYTES_PER_LINE - 1 - (self.cursor.head % BYTES_PER_LINE);
				self.cursor.collapse();
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char('w') => {
				self.cursor.move_to_next_word(self.contents.len() - 1);
				self.clamp_screen_to_cursor();
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char('e') => {
				self.cursor.move_to_next_end(self.contents.len() - 1);
				self.clamp_screen_to_cursor();
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char('b') => {
				self.cursor.move_to_previous_beginning();
				self.clamp_screen_to_cursor();
			}
			
			(Mode::Normal, Event::Key(key_event), None)
			   if key_event.code == KeyCode::Char(';') => {
				self.cursor.collapse();
			}
			
			(Mode::Normal, Event::Key(_), Some(_)) => {
				self.partial_action = None;
			}
			
			_ => {}
		}
	}
	
	const fn clamp_screen_to_cursor(&mut self) {
		if self.cursor.head < self.scroll_position {
			self.scroll_position -= (self.scroll_position - self.cursor.head).next_multiple_of(BYTES_PER_LINE);
		} else if self.cursor.head > self.scroll_position + self.screen_size() - 1 {
			let screen_edge_offset_to_cursor = self.cursor.head - (self.scroll_position + self.screen_size() - 1);
			self.scroll_position += screen_edge_offset_to_cursor.next_multiple_of(BYTES_PER_LINE);
		}
	}
}

const fn previous_multiple_of(multiple: usize, number: usize) -> usize {
	if number == 0 {
		0
	} else {
		(number - 1) - ((number - 1) % multiple)
	}
}
