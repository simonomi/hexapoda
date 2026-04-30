use std::{io, path::PathBuf, process::exit, time::Duration};
use crossterm::{ExecutableCommand, event::{self, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind}, terminal::window_size};
use ratatui::{DefaultTerminal, style::Stylize, text::Span};
use crate::{BYTES_PER_LINE, action::AppAction, buffer::Buffer, config::{Config, ConfigInitError}, cursor::Cursor, window_size::WindowSize};

mod widget;
mod actions;

pub struct App {
	pub config: Config,
	
	pub buffers: Vec<Buffer>,
	pub current_buffer_index: usize,
	
	pub primary_cursor_register: Vec<u8>,
	pub other_cursor_registers: Vec<Vec<u8>>,
	
	pub window_size: WindowSize,
	
	pub should_quit: bool,
	
	pub is_dragging_mouse: bool,
	
	pub logs: Vec<String>,
}

impl App {
	pub fn new(
		config_path: Option<PathBuf>,
		files: &[PathBuf],
	) -> Self {
		let config = {
			let config = Config::init(config_path);
			
			match &config {
				Err(ConfigInitError::IO(io_error)) if io_error.kind() != io::ErrorKind::NotFound => {
					eprintln!("IO error while reading config, press <ENTER> to continue with default config");
					
					let mut temp = String::new();
					let _ = io::stdin().read_line(&mut temp);
				}
				Err(ConfigInitError::Deserialization(deserialization_error)) => {
					eprintln!("bad config: {deserialization_error}");
					eprintln!("press <ENTER> to continue with default config");
					
					let mut temp = String::new();
					let _ = io::stdin().read_line(&mut temp);
				}
				_ => {}
			}
			
			config.unwrap_or_default()
		};
		
		let mut error_alert: Option<Span> = None;
		
		let mut buffers: Vec<Buffer> = files
			.iter()
			.filter_map(|path| {
				Buffer::from_file_at(path.clone())
					.inspect_err(|error| {
						error_alert = Some(
							Span::raw(format!("error reading '{}': {error}", path.display())).red()
						);
					})
					.ok()
			})
			.collect();
		
		if files.is_empty() {
			#[cfg(target_os = "macos")] {
				eprintln!("please provide at least one file as input. use --help for options");
				exit(1);
			}
			
			#[cfg(not(target_os = "macos"))] {
				use io::{Read, stdin};
				
				let mut standard_input = Vec::new();
				stdin().read_to_end(&mut standard_input).unwrap();
				buffers.push(Buffer::new("-".into(), standard_input));
			}
		}
		
		if let Some(error_alert) = error_alert {
			if buffers.is_empty() {
				eprintln!("{error_alert}");
				exit(1);
			} else {
				buffers[0].alert_message = error_alert;
			}
		}
		
		let window_size = WindowSize {
			rows: window_size().unwrap().rows as usize,
			covered_rows: if buffers.len() > 1 {
				2 // status line and tab bar
			} else {
				1 // status line
			},
		};
		
		Self {
			config,
			
			buffers,
			current_buffer_index: 0,
			
			primary_cursor_register: Vec::new(),
			other_cursor_registers: Vec::new(),
			
			window_size,
			
			should_quit: false,
			
			is_dragging_mouse: false,
			
			logs: Vec::new(),
		}
	}
	
	pub fn handle_events(&mut self, terminal: &mut DefaultTerminal) -> bool {
		let mut should_redraw = self.handle_event(terminal);
		
		while event::poll(Duration::ZERO).unwrap() {
			should_redraw |= self.handle_event(terminal);
		}
		
		should_redraw
	}
	
	pub fn handle_event(&mut self, terminal: &mut DefaultTerminal) -> bool {
		let event = event::read()
			.inspect_err(|error| {
				#[cfg(target_os = "macos")] {
					use io::ErrorKind;

					if error.kind() == ErrorKind::Other {
						let error_message = error.to_string();
						if error_message == "Failed to initialize input reader" {
							eprintln!("reading from stdin on macOS does not work due to a limitation in crossterm. see https://github.com/crossterm-rs/crossterm/issues/396");
							exit(1);
						}
					}
				}
			})
			.unwrap();
		
		// self.logs.push(format!("{event:?}"));
		
		match event {
			Event::Resize(_, height) => {
				self.window_size.rows = height as usize;
			
				self.buffers[self.current_buffer_index]
					.clamp_screen_to_primary_cursor(self.window_size);
				
				true
			}
			Event::Key(key_event) => self.handle_key(key_event, terminal),
			Event::Mouse(mouse_event) => self.handle_mouse(mouse_event),
			_ => false
		}
	}
	
	fn handle_key(&mut self, key_event: KeyEvent, terminal: &mut DefaultTerminal) -> bool {
		if key_event.modifiers == KeyModifiers::CONTROL &&
		   key_event.code == KeyCode::Char('c')
		{
			terminal.backend_mut().execute(DisableMouseCapture).unwrap();
			crossterm::terminal::disable_raw_mode().unwrap();
			ratatui::restore();
			exit(130);
		}
		
		let (maybe_app_action, should_redraw) = self.buffers[self.current_buffer_index].handle_key(
			key_event,
			&self.config,
			&self.primary_cursor_register,
			&self.other_cursor_registers,
			self.window_size
		);
		
		if let Some(app_action) = maybe_app_action {
			match app_action {
				AppAction::QuitIfSaved => self.quit_if_saved(),
				AppAction::Quit => self.quit(),
				
				AppAction::PreviousBuffer => self.previous_buffer(),
				AppAction::NextBuffer => self.next_buffer(),
				
				AppAction::Yank => self.yank(),
			}
		}
		
		should_redraw || maybe_app_action.is_some()
	}
	
	fn handle_mouse(&mut self, mouse_event: MouseEvent) -> bool {
		let position = self.mouse_event_position(mouse_event);
		let current_buffer = &mut self.buffers[self.current_buffer_index];
		
		match mouse_event.kind {
			MouseEventKind::Down(_) => {
				if let Some(position) = position {
					current_buffer.primary_cursor = Cursor::at(position);
					current_buffer.cursors.clear();
					current_buffer.clamp_screen_to_primary_cursor(self.window_size);
					self.is_dragging_mouse = true;
				}
				
				true
			},
			MouseEventKind::Drag(_) if self.is_dragging_mouse => {
				if let Some(position) = position {
					current_buffer.primary_cursor.head = position;
					current_buffer.clamp_screen_to_primary_cursor(self.window_size);
				}
				
				true
			},
			MouseEventKind::Up(_) if self.is_dragging_mouse => {
				if let Some(position) = position {
					current_buffer.primary_cursor.head = position;
					current_buffer.clamp_screen_to_primary_cursor(self.window_size);
				}
				
				self.is_dragging_mouse = false;
				
				true
			},
			MouseEventKind::ScrollDown => {
				for _ in 0..3 {
					current_buffer.scroll_down(self.window_size);
				}
				
				true
			},
			MouseEventKind::ScrollUp => {
				for _ in 0..3 {
					current_buffer.scroll_up(self.window_size);
				}
				
				true
			},
			_ => false,
		}
	}
	
	fn mouse_event_position(&self, mouse_event: MouseEvent) -> Option<usize> {
		let tab_bar_rows = usize::from(self.buffers.len() > 1);
		
		if usize::from(mouse_event.row) - tab_bar_rows >= self.window_size.hex_rows() {
			return None;
		}
		
		let current_buffer = &self.buffers[self.current_buffer_index];
		
		let byte_column = match mouse_event.column {
			10..=11 => Some(0),
			13..=14 => Some(1),
			16..=17 => Some(2),
			19..=20 => Some(3),
			
			23..=24 => Some(4),
			26..=27 => Some(5),
			29..=30 => Some(6),
			32..=33 => Some(7),
			
			36..=37 => Some(8),
			39..=40 => Some(9),
			42..=43 => Some(10),
			45..=46 => Some(11),
			
			49..=50 => Some(12),
			52..=53 => Some(13),
			55..=56 => Some(14),
			58..=59 => Some(15),
			
			_ => None,
		};
		
		byte_column.map(|byte_column| {
			current_buffer.scroll_position +
			(mouse_event.row as usize - tab_bar_rows) * BYTES_PER_LINE +
			byte_column
		})
		
		// if let Some(byte_column) = byte_column &&
		// 	mouse_event.row as usize - tab_bar_rows < self.window_size.hex_rows()
		// {
		// 	Some(
		// 		current_buffer.scroll_position +
		// 		(mouse_event.row as usize - tab_bar_rows) * BYTES_PER_LINE +
		// 		byte_column
		// 	)
		// } else {
		// 	None
		// }
	}
}
