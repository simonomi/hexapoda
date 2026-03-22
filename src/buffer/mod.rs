use core::slice::GetDisjointMutIndex;
use std::{collections::HashSet, fs::File, io::Read, path::PathBuf};
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, style::{Color, Stylize}, text::Span, widgets::Widget};
use crate::{BYTES_PER_LINE, action::{Action, AppAction, bytes_to_nat}, app::WindowSize, config::Config, cursor::Cursor, edit_action::EditAction};

mod widget;

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
	
	pub inspecting_selection: bool,
	
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
	Goto, View, Replace, Space, Repeat, To
}

#[derive(Clone)]
pub struct Popup {
	at: usize,
	width: u16,
	lines: Vec<String>,
	color: Option<[u8; 3]>
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
			Self::Repeat => "×",
			Self::To => "t",
		}
	}
}

impl Popup {
	pub fn new(at: usize, lines: Vec<String>, color: Option<[u8; 3]>) -> Self {
		Self {
			at,
			width: lines
				.iter()
				.map(|line| line.len() as u16)
				.chain(color.map(|_| 7))
				.max()
				.unwrap_or(0),
			lines,
			color
		}
	}
	
	fn area_at(&self, x: u16, y: u16) -> Rect {
		Rect {
			x,
			y,
			width: self.width + 2,
			height: self.lines.len() as u16 + u16::from(self.color.is_some())
		}
	}
}

impl Widget for Popup {
	fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
		let width = self.width as usize;
		
		for (line, area) in self.lines.iter().zip(area.rows()) {
			Span::from(format!(" {line:^width$} "))
				.white()
				.on_dark_gray()
				.render(area, buf);
		}
		
		if let Some([red, green, blue]) = self.color {
			let color_code = format!("#{red:02X}{green:02X}{blue:02X}");
			
			Span::from(format!(" {color_code:^width$} "))
				.fg(Color::Rgb(red, green, blue))
				.on_dark_gray()
				.render(area.rows().next_back().unwrap(), buf);
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
			primary_cursor: Cursor::default(),
			cursors: Vec::new(),
			
			marks: HashSet::new(),
			
			mode: Mode::Normal,
			partial_action: None,
			partial_replace: None,
			
			alert_message: "".into(),
			popups: Vec::new(),
			
			inspecting_selection: false,
			
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
	) -> Option<AppAction> {
		self.alert_message = "".into();
		self.popups.clear();
		
		let was_inspecting_selection = self.inspecting_selection;
		
		let app_action = match self.partial_action {
			Some(PartialAction::Replace) => {
				self.handle_replace(event, window_size);
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
				None
			},
			_ => self.handle_other_modes(event, config, window_size),
		};
		
		if was_inspecting_selection {
			self.inspecting_selection = false;
		}
		
		assert!(self.scroll_position.is_multiple_of(BYTES_PER_LINE));
		assert!(self.scroll_position < self.contents.len());
		assert!(self.primary_cursor.head < self.contents.len());
		assert!(self.primary_cursor.tail < self.contents.len());
		assert!(self.scroll_position <= self.primary_cursor.head);
		assert!(self.primary_cursor.head < self.scroll_position + window_size.visible_byte_count());
		
		app_action
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
	) -> Option<AppAction> {
		let mut result = None;
		
		let should_reset_partial = self.partial_action.is_some();
		
		if let Some(mode_config) = config.0.get(&self.mode) &&
		   let Some(keybinds) = mode_config.0.get(&self.partial_action) &&
		   let Some(action) = keybinds.0.get(&event.into())
		{
			match action {
				Action::App(app_action) => result = Some(*app_action),
				Action::Buffer(buffer_action) => self.execute(*buffer_action, window_size),
				Action::Cursor(cursor_action) => {
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
		}
		
		if should_reset_partial {
			self.partial_action = None;
		}
		
		result
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
				_ => panic!("repeated actions may only be cursor actions"),
			}
		}
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
		
		while !self.cursors.is_empty() && index < self.cursors.len() {
			while index < self.cursors.len() - 1 &&
				self.cursors[index].range().is_overlapping(
					&self.cursors[index + 1].range())
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
			}
			
			index += 1;
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
			assert_eq!(nybble_from_hex(character), Some(index as u8));
		}
	}
}
