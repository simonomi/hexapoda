use std::{fs::File, io::Read, path::PathBuf};
use crossterm::event::KeyEvent;
use ratatui::{style::Color, text::Span};
use crate::{action::AppAction, app::WindowSize, config::Config, cursor::Cursor, edit_action::EditAction};

mod widget;

pub struct Buffer {
	pub file_name: String,
	pub file_path: PathBuf,
	
	pub contents: Vec<u8>,
	
	pub scroll_position: usize,
	pub cursor: Cursor,
	
	pub mode: Mode,
	pub partial_action: Option<PartialAction>,
	pub partial_replace: Option<u8>,
	
	pub alert_message: Span<'static>,
	
	pub edit_history: Vec<EditAction>,
	// the index *after* the latest edit action
	pub time_traveling: Option<usize>,
	// the index *after* the last saved edit action
	pub last_saved_at: Option<usize>,
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

impl Buffer {
	pub fn new(file_path: PathBuf) -> Self {
		let file = File::open(&file_path);
		let mut contents = Vec::new();
		file.unwrap().read_to_end(&mut contents).unwrap();
		
		Self {
			file_name: file_path.file_name().unwrap().to_str().unwrap().to_owned(),
			file_path,
			
			contents,
			
			scroll_position: 0,
			cursor: Cursor::default(),
			
			mode: Mode::Normal,
			partial_action: None,
			partial_replace: None,
			
			alert_message: "".into(),
			
			edit_history: Vec::new(),
			time_traveling: None,
			last_saved_at: Some(0),
		}
	}
	
	pub fn handle_key(
		&mut self,
		event: KeyEvent,
		config: &Config,
		window_size: WindowSize
	) -> Option<AppAction> {
		self.alert_message = "".into();
		
		let mut app_action = None;
		
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
			
			if let Some(mode_config) = config.0.get(&self.mode) &&
			   let Some(keybinds) = mode_config.0.get(&self.partial_action) &&
			   let Some(action) = keybinds.0.get(&event.into())
			{
				app_action = self.execute(*action, window_size);
			}
			
			if should_reset_partial {
				self.partial_action = None;
			}
		}
		
		app_action
	}
	
	pub const fn has_unsaved_changes(&self) -> bool {
		!self.all_changes_saved()
	}
	
	pub const fn all_changes_saved(&self) -> bool {
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
	
	// returns 0 if empty
	pub const fn max_contents_index(&self) -> usize {
		self.contents.len().saturating_sub(1)
	}
}

fn nybble_from_hex(hex: char) -> Option<u8> {
	if !hex.is_ascii() { return None; }
	
	match hex {
		'0'..='9' => Some(u8::try_from(hex).unwrap() - u8::try_from('0').unwrap()),
		'a'..='f' => Some(u8::try_from(hex).unwrap() - u8::try_from('a').unwrap() + 10),
		'A'..='F' => Some(u8::try_from(hex).unwrap() - u8::try_from('A').unwrap() + 10),
		_ => None
	}
}

mod tests {
	#[allow(unused_imports)]
	use crate::buffer::nybble_from_hex;
	
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
