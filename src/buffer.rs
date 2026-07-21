use std::{collections::HashSet, fs::File, io::{self, Read}, path::{Path, PathBuf}};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Position, style::Stylize, text::Span};
use serde::{Deserialize, Serialize};
use crate::{BYTES_PER_LINE, action::{Action, AppAction}, buffer::actions::bytes_to_nat, config::Config, cursor::Cursor, edit_action::EditAction, popup::Popup, utilities::IsOverlapping, window_size::WindowSize};

mod widget;
mod actions;

pub struct Buffer {
	pub file_name: String,
	pub file_path: PathBuf,
	
	pub contents: Vec<u8>,
	
	pub scroll_position: usize,
	pub primary_cursor: Cursor,
	pub cursors: Vec<Cursor>,
	
	pub marks: HashSet<usize>,
	
	pub mode: Mode,
	pub partial_action: Option<PartialAction>,
	pub partial_replace: Option<u8>,
	
	pub alert_message: Span<'static>,
	pub popups: Vec<Popup>,
	
	// used for `go`, `/`, `A-/`, etc
	pub entry_text: String,
	pub entry_cursor_index: usize,
	// where on the screen the cursor is rendered
	pub cursor_position: Option<Position>,
	
	pub inspection_status: Option<InspectionStatus>,
	
	pub edit_history: Vec<EditAction>,
	// the index *after* the latest edit action
	pub time_traveling: Option<usize>,
	// the index *after* the last saved edit action
	pub last_saved_at: Option<usize>,
	
	pub logs: Vec<String>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Debug)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
	Normal, Select, // Insert
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Debug)]
#[serde(rename_all = "snake_case")]
pub enum PartialAction {
	Goto, View, Replace, Space, Repeat, Till, GotoOffset, GotoDecimalOffset
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InspectionStatus {
	Normal, ColorsOnly
}

impl TryFrom<&str> for PartialAction {
	type Error = ();
	
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		use PartialAction::*;
		
		match value {
			"goto" => Ok(Goto),
			"view" => Ok(View),
			"replace" => Ok(Replace),
			"space" => Ok(Space),
			"repeat" => Ok(Repeat),
			"till" => Ok(Till),
			_ => Err(()),
		}
	}
}

impl Buffer {
	pub fn from_file_at(file_path: &Path) -> io::Result<Self> {
		let file_path = file_path.canonicalize()?;
		
		let mut file = File::open(&file_path)?;
		let mut contents = Vec::new();
		file.read_to_end(&mut contents)?;
		
		Ok(Self::new(file_path, contents))
	}
	
	pub fn new(file_path: PathBuf, contents: Vec<u8>) -> Self {
		Self {
			file_name: file_path.file_name().unwrap().to_str().unwrap().to_owned(),
			file_path,
			
			contents,
			
			scroll_position: 0,
			primary_cursor: Cursor::default(),
			cursors: Vec::new(),
			
			marks: HashSet::new(),
			
			mode: Mode::Normal,
			partial_action: None,
			partial_replace: None,
			
			alert_message: "".into(),
			popups: Vec::new(),
			
			entry_text: String::new(),
			entry_cursor_index: 0,
			cursor_position: None,
			
			inspection_status: None,
			
			edit_history: Vec::new(),
			time_traveling: None,
			last_saved_at: Some(0),
			
			logs: Vec::new(),
		}
	}
	
	pub fn handle_key(
		&mut self,
		event: KeyEvent,
		config: &Config,
		primary_cursor_register: &[u8],
		other_cursor_registers: &[Vec<u8>],
		window_size: WindowSize
	) -> (Option<AppAction>, bool) {
		let mut should_redraw = !self.alert_message.content.is_empty();
		self.alert_message = "".into();
		// self.logs.push(format!("{event:?}"));
		
		let app_action = match self.partial_action {
			Some(PartialAction::Replace) => {
				self.handle_replace(event, window_size);
				should_redraw = true;
				None
			},
			Some(PartialAction::Repeat) => {
				self.handle_repeat(
					event,
					config,
					primary_cursor_register,
					other_cursor_registers,
					window_size
				);
				should_redraw = true;
				None
			},
			Some(PartialAction::GotoOffset) => {
				self.handle_goto_offset(event, window_size);
				should_redraw = true;
				None
			}
			Some(PartialAction::GotoDecimalOffset) => {
				self.handle_goto_decimal_offset(event, window_size);
				should_redraw = true;
				None
			}
			_ => {
				let (app_action, redraw) = self.handle_other_modes(event, config, window_size);
				should_redraw |= redraw;
				app_action
			},
		};
		
		assert!(self.scroll_position.is_multiple_of(BYTES_PER_LINE));
		if !self.contents.is_empty() {
			assert!(self.scroll_position < self.contents.len());
			assert!(self.primary_cursor.head < self.contents.len());
			assert!(self.primary_cursor.tail < self.contents.len());
		}
		assert!(self.scroll_position <= self.primary_cursor.head);
		assert!(self.primary_cursor.head < self.scroll_position + window_size.visible_byte_count());
		
		debug_assert!(self.cursors.is_sorted_by_key(|cursor| cursor.head));
		
		(app_action, should_redraw)
	}
	
	fn handle_replace(&mut self, event: KeyEvent, window_size: WindowSize) {
		if let Some(hex_character) = event.code.as_char() &&
		   let Some(nybble) = nybble_from_hex(hex_character)
		{
			if let Some(partial_replace) = self.partial_replace.take() {
				self.execute_and_add(
					EditAction::Replace {
						primary_cursor: self.primary_cursor,
						cursors: self.cursors.clone(),
						primary_old_data: self.contents[self.primary_cursor.range()].to_vec(),
						old_data: self.cursors
							.iter()
							.map(|cursor| self.contents[cursor.range()].to_vec())
							.collect(),
						new_byte: partial_replace << 4 | nybble
					},
					window_size
				);
				self.partial_action = None;
			} else {
				self.partial_replace = Some(nybble);
			}
		} else {
			self.partial_action = None;
			self.partial_replace = None;
		}
	}
	
	fn handle_other_modes(
		&mut self,
		event: KeyEvent,
		config: &Config,
		window_size: WindowSize
	) -> (Option<AppAction>, bool) {
		use Action::*;
		
		let mut result = None;
		
		let initial_partial_action = self.partial_action;
		let mut should_redraw = self.partial_action.is_some();
		
		if let Some(mode_config) = config.0.get(&self.mode) &&
		   let Some(keybinds) = mode_config.0.get(&self.partial_action) &&
		   let Some(action) = keybinds.0.get(&event.into())
		{
			should_redraw = true;
			
			if action.clears_popups() {
				self.popups.clear();
			}
			
			match action {
				App(app_action) => result = Some(*app_action),
				Buffer(buffer_action) => self.execute(*buffer_action, window_size),
				Cursor(cursor_action) => {
					let max_contents_index = self.max_contents_index();
					
					self.primary_cursor.execute(*cursor_action, max_contents_index);
					
					for cursor in &mut self.cursors {
						cursor.execute(*cursor_action, max_contents_index);
					}
					self.cursors.sort_by_key(|cursor| cursor.head);
					
					self.combine_cursors_if_overlapping();
					self.clamp_screen_to_primary_cursor(window_size);
				},
			}
			
			if action.clears_popups() && !action.is_inspection() {
				self.inspection_status = None;
			}
		}
		
		if self.partial_action == initial_partial_action {
			self.partial_action = None;
		}
		
		(result, should_redraw)
	}
	
	fn handle_repeat(
		&mut self,
		event: KeyEvent,
		config: &Config,
		primary_cursor_register: &[u8],
		other_cursor_registers: &[Vec<u8>],
		window_size: WindowSize
	) {
		self.partial_action = None;
		
		if let Some(mode_config) = config.0.get(&self.mode) &&
		   let Some(keybinds) = mode_config.0.get(&Some(PartialAction::Repeat)) &&
		   let Some(action) = keybinds.0.get(&event.into())
		{
			match action {
				Action::Cursor(cursor_action) => {
					let Some(primary_repeat_count) = bytes_to_nat(primary_cursor_register) else {
						self.alert_message = Span::from(
							"repeat count is too large"
						).red();
						return;
					};
					let other_repeat_counts = other_cursor_registers
						.iter()
						.map(|register| bytes_to_nat(register));
					
					if other_repeat_counts.clone().any(|count| count.is_none()) {
						self.alert_message = Span::from(
							"repeat count is too large"
						).red();
						return;
					}
					
					let max_contents_index = self.max_contents_index();
					
					for _ in 0..primary_repeat_count {
						self.primary_cursor.execute(*cursor_action, max_contents_index);
					}
					
					for (cursor, repeat_count) in self.cursors.iter_mut().zip(other_repeat_counts) {
						for _ in 0..repeat_count.unwrap() {
							cursor.execute(*cursor_action, max_contents_index);
						}
					}
					self.cursors.sort_by_key(|cursor| cursor.head);
					
					self.combine_cursors_if_overlapping();
					self.clamp_screen_to_primary_cursor(window_size);
				},
				_ => {
					self.alert_message = Span::from(
						"only cursor actions may be repeated"
					).red();
				}
			}
		}
	}
	
	fn handle_goto_offset(
		&mut self,
		event: KeyEvent,
		window_size: WindowSize
	) {
		if let Some(hex_character) = event.code.as_char() &&
		   let Some(_) = nybble_from_hex(hex_character)
		{
			self.entry_text.insert(self.entry_cursor_index, hex_character);
			self.entry_cursor_index += 1;
		} else {
			match event.code {
				KeyCode::Backspace => {
					if self.entry_cursor_index > 0 {
						self.entry_text.remove(self.entry_cursor_index - 1);
						self.entry_cursor_index -= 1;
					}
				}
				KeyCode::Delete => {
					if self.entry_cursor_index < self.entry_text.len() {
						self.entry_text.remove(self.entry_cursor_index);
					}
				}
				KeyCode::Left => {
					if self.entry_cursor_index > 0 {
						self.entry_cursor_index -= 1;
					}
				}
				KeyCode::Right => {
					if self.entry_cursor_index < self.entry_text.len() {
						self.entry_cursor_index += 1;
					}
				}
				KeyCode::Enter => {
					// entry_text should always be 0-9a-fA-F
					let entered_offset = usize::from_str_radix(&self.entry_text, 16).unwrap();
					
					if entered_offset < self.contents.len() {
						self.primary_cursor = Cursor::at(entered_offset);
						self.cursors.clear();
						
						self.clamp_screen_to_primary_cursor(window_size);
					} else {
						self.alert_message = Span::from(
							"offset out of bounds"
						).red();
					}
					
					self.partial_action = None;
				}
				_ => {
					self.partial_action = None;
				}
			}
		}
		
		self.cursor_position = self.partial_action.is_some()
			.then(|| Position {
				x: u16::try_from(self.entry_cursor_index).unwrap() + 9, // length of entry label
				y: u16::try_from(window_size.rows).unwrap() - 2
			});
	}
	
	fn handle_goto_decimal_offset(
		&mut self,
		event: KeyEvent,
		window_size: WindowSize
	) {
		if let Some(hex_character) = event.code.as_char() &&
		   hex_character.is_ascii_digit()
		{
			self.entry_text.insert(self.entry_cursor_index, hex_character);
			self.entry_cursor_index += 1;
		} else {
			match event.code {
				KeyCode::Backspace => {
					if self.entry_cursor_index > 0 {
						self.entry_text.remove(self.entry_cursor_index - 1);
						self.entry_cursor_index -= 1;
					}
				}
				KeyCode::Delete => {
					if self.entry_cursor_index < self.entry_text.len() {
						self.entry_text.remove(self.entry_cursor_index);
					}
				}
				KeyCode::Left => {
					if self.entry_cursor_index > 0 {
						self.entry_cursor_index -= 1;
					}
				}
				KeyCode::Right => {
					if self.entry_cursor_index < self.entry_text.len() {
						self.entry_cursor_index += 1;
					}
				}
				KeyCode::Enter => {
					// entry_text should always be 0-9a-fA-F
					let entered_offset = self.entry_text.parse().unwrap();
					
					if entered_offset < self.contents.len() {
						self.primary_cursor = Cursor::at(entered_offset);
						self.cursors.clear();
						
						self.clamp_screen_to_primary_cursor(window_size);
					} else {
						self.alert_message = Span::from(
							"offset out of bounds"
						).red();
					}
					
					self.partial_action = None;
				}
				_ => {
					self.partial_action = None;
				}
			}
		}
		
		self.cursor_position = self.partial_action.is_some()
			.then(|| Position {
				x: u16::try_from(self.entry_cursor_index).unwrap() + 7, // length of entry label
				y: u16::try_from(window_size.rows).unwrap() - 2
			});
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
	
	pub fn combine_cursors_if_overlapping(&mut self) {
		let mut index = 0;
		
		// TODO: this can miss some in the WEIRD case that
		// [    *]
		//           [    *]
		//   [                 *]
		// where * is the head.
		// the first one wont merge with the 2nd, but the 2nd will
		// merge with the 3rd, which would then overlap with the 1st,
		// but won't be checked
		
		while !self.cursors.is_empty() && index < self.cursors.len() {
			while index < self.cursors.len() - 1 &&
				self.cursors[index].range().is_overlapping(
					&self.cursors[index + 1].range()
				)
			{
				let next_cursor = self.cursors[index + 1];
				self.cursors[index].combine_with(next_cursor);
				self.cursors.remove(index + 1);
			}
			
			if self.primary_cursor.range()
				.is_overlapping(&self.cursors[index].range())
			{
				self.primary_cursor.combine_with(self.cursors[index]);
				self.cursors.remove(index);
			} else {
				index += 1;
			}
		}
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
			assert_eq!(nybble_from_hex(character), Some(u8::try_from(index).unwrap()));
		}
	}
}
