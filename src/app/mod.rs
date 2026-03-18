use std::{env, fs::File, io::Read, path::PathBuf, process::exit};
use crossterm::{event::{self, Event, KeyEvent}, terminal::window_size};
use ratatui::style::Color;
use crate::{config::Config, cursor::Cursor, edit_action::EditAction};

mod widget;

pub struct App {
	pub config: Config,
	pub file_name: String,
	pub file_path: PathBuf,
	
	pub contents: Vec<u8>,
	
	pub window_rows: usize,
	
	pub scroll_position: usize,
	pub cursor: Cursor,
	
	pub should_quit: bool,
	
	pub mode: Mode,
	pub partial_action: Option<PartialAction>,
	pub partial_replace: Option<u8>,
	
	pub edit_history: Vec<EditAction>,
	// the index *after* the latest edit action
	pub time_traveling: Option<usize>,
	// the index *after* the last saved edit action
	pub last_saved_at: Option<usize>,
	
	pub logs: Vec<String>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Mode {
	Normal, Select, Insert
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum PartialAction {
	Goto, View, Replace, Space
}

impl Mode {
	pub const fn label(self) -> &'static str {
		match self {
			Self::Normal => " NORMAL ",
			Self::Select => " SELECT ",
			Self::Insert => " INSERT ",
		}
	}
	
	pub const fn color(self) -> Color {
		match self {
			Self::Normal => Color::Blue,
			Self::Select => Color::Yellow,
			Self::Insert => Color::Green,
		}
	}
}

impl PartialAction {
	pub const fn label(self) -> &'static str {
		match self {
			Self::Goto => "g",
			Self::View => "z",
			Self::Replace => "r",
			Self::Space => "␠",
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
			config: Config::default(),
			file_name: file_path.file_name().unwrap().to_str().unwrap().to_owned(),
			file_path,
			
			contents,
			
			// -1 because of the status line
			window_rows: window_size().unwrap().rows as usize - 1,
			
			scroll_position: 0,
			cursor: Cursor::default(),
			
			should_quit: false,
			
			mode: Mode::Normal,
			partial_action: None,
			partial_replace: None,
			
			edit_history: Vec::new(),
			time_traveling: None,
			last_saved_at: Some(0),
			
			logs: Vec::new(),
		}
	}
	
	#[allow(clippy::too_many_lines)]
	pub fn handle_events(&mut self) {
		#[allow(clippy::collapsible_match)]
		match event::read().unwrap() {
			Event::Resize(_, height) => {
				// -1 because of the status line
				self.window_rows = height as usize - 1;
			}
			Event::Key(key_event) => self.handle_key(key_event),
			// Event::Mouse(mouse_event) => {
			// 	mouse_event.kind
			// },
			_ => {}
		}
	}
	
	fn handle_key(&mut self, event: KeyEvent) {
		if self.partial_action == Some(PartialAction::Replace) {
			if let Some(hex_character) = event.code.as_char() &&
			   let Some(nybble) = nybble_from_hex(hex_character)
			{
				if let Some(partial_replace) = self.partial_replace.take() {
					self.execute_and_add(
						EditAction::Replace {
							cursor: self.cursor,
							old_data: self.contents[self.cursor.range()].into(),
							new_byte: partial_replace << 4 | nybble
						}
					);
					self.partial_action = None;
				} else {
					self.partial_replace = Some(nybble);
				}
			} else {
				self.partial_action = None;
				self.partial_replace = None;
			}
		} else {
			let should_reset_partial = self.partial_action.is_some();
			
			if let Some(mode_config) = self.config.0.get(&self.mode) &&
			   let Some(keybinds) = mode_config.0.get(&self.partial_action) &&
			   let Some(action) = keybinds.0.get(&event.into())
			{
				self.execute(*action);
			}
			
			if should_reset_partial {
				self.partial_action = None;
			}
		}
	}
	
	const fn has_unsaved_changes(&self) -> bool {
		!self.all_changes_saved()
	}
	
	const fn all_changes_saved(&self) -> bool {
		if let Some(last_saved_at) = self.last_saved_at {
			if let Some(time_traveling) = self.time_traveling {
				last_saved_at == time_traveling
			} else {
				last_saved_at == self.edit_history.len()
			}
		} else {
			false
		}
	}
}

fn nybble_from_hex(hex: char) -> Option<u8> {
	if !hex.is_ascii() { return None }
	
	match hex {
		'0'..='9' => Some(u8::try_from(hex).unwrap() - u8::try_from('0').unwrap()),
		'a'..='f' => Some(u8::try_from(hex).unwrap() - u8::try_from('a').unwrap() + 10),
		'A'..='F' => Some(u8::try_from(hex).unwrap() - u8::try_from('A').unwrap() + 10),
		_ => None
	}
}

mod tests {
	#[allow(unused_imports)]
	use crate::app::nybble_from_hex;
	
	#[test]
	fn nybble_from_hex_case_doesnt_matter() {
		for character in 'a'..='f' {
			assert_eq!(nybble_from_hex(character), nybble_from_hex(character.to_ascii_uppercase()));
		}
	}
	
	#[test]
	fn nybble_from_hex_digits_are_correct() {
		for (index, character) in ('0'..='9').enumerate() {
			assert_eq!(nybble_from_hex(character), Some(index as u8));
		}
	}
}
