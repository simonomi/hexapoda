use std::{env, fs::File, io::Read, path::PathBuf, process::exit};
use crossterm::{event::{self, Event, KeyEvent}, terminal::window_size};
use ratatui::style::Color;
use crate::{config::Config, cursor::Cursor, edit_action::EditAction};

mod widget;

pub struct App {
	pub config: Config,
	pub file_name: String,
	
	pub contents: Vec<u8>,
	
	pub window_rows: usize,
	
	pub scroll_position: usize,
	pub cursor: Cursor,
	
	pub should_quit: bool,
	
	pub mode: Mode,
	pub partial_action: Option<PartialAction>,
	
	pub edit_history: Vec<EditAction>,
	// some index to keep track of where we are? edit_prophecy?
	
	pub logs: Vec<String>,
}

#[derive(Hash, PartialEq, Eq)]
pub enum Mode {
	Normal, Select, Insert
}

#[derive(Hash, PartialEq, Eq)]
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
			config: Config::default(),
			file_name: file_path.file_name().unwrap().to_str().unwrap().to_owned(),
			
			contents,
			
			// -1 because of the status line
			window_rows: window_size().unwrap().rows as usize - 1,
			
			scroll_position: 0,
			cursor: Cursor::default(),
			
			should_quit: false,
			
			mode: Mode::Normal,
			partial_action: None,
			
			edit_history: Vec::new(),
			
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
