use std::{env, process::exit, time::Duration};
use crossterm::{ExecutableCommand, event::{self, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind}, terminal::window_size};
use ratatui::{DefaultTerminal, style::Stylize, text::Span};
use crate::{BYTES_PER_LINE, action::AppAction, buffer::Buffer, config::Config, cursor::Cursor};

mod widget;

pub struct App {
	pub config: Config,
	
	pub buffers: Vec<Buffer>,
	pub current_buffer_index: usize,
	
	pub primary_cursor_register: Vec<u8>,
	pub other_cursor_registers: Vec<Vec<u8>>,
	
	pub window_size: WindowSize,
	
	pub should_quit: bool,
	
	pub logs: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct WindowSize {
	pub rows: usize,
	pub covered_rows: usize,
}

impl App {
	pub fn new() -> Self {
		let buffers: Vec<Buffer> = env::args()
				.skip(1)
				.map(Into::into)
				.map(Buffer::new)
				.collect();
		
		if buffers.is_empty() {
			println!("please provide at least one file as input");
			exit(1);
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
			config: Config::default(),
			
			buffers,
			current_buffer_index: 0,
			
			primary_cursor_register: Vec::new(),
			other_cursor_registers: Vec::new(),
			
			window_size,
			
			should_quit: false,
			
			logs: Vec::new(),
		}
	}
	
	pub fn handle_events(&mut self, terminal: &mut DefaultTerminal) {
		self.handle_event(terminal);
		
		while event::poll(Duration::ZERO).unwrap() {
			self.handle_event(terminal);
		}
	}
	
	pub fn handle_event(&mut self, terminal: &mut DefaultTerminal) {
		match event::read().unwrap() {
			Event::Resize(_, height) => {
				self.window_size.rows = height as usize;
			
				self.buffers[self.current_buffer_index]
					.clamp_screen_to_primary_cursor(self.window_size);
			}
			Event::Key(key_event) => self.handle_key(key_event, terminal),
			Event::Mouse(mouse_event) => self.handle_mouse(mouse_event),
			_ => {}
		}
	}
	
	fn handle_key(&mut self, key_event: KeyEvent, terminal: &mut DefaultTerminal) {
		if key_event.modifiers == KeyModifiers::CONTROL &&
		   key_event.code == KeyCode::Char('c')
		{
			terminal.backend_mut().execute(DisableMouseCapture).unwrap();
			crossterm::terminal::disable_raw_mode().unwrap();
			ratatui::restore();
			exit(130);
		}
		
		let maybe_app_action = self.buffers[self.current_buffer_index].handle_key(
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
	}
	
	fn handle_mouse(&mut self, mouse_event: MouseEvent) {
		let tab_bar_rows = usize::from(self.buffers.len() > 1);
		let current_buffer = &mut self.buffers[self.current_buffer_index];
		
		match mouse_event.kind {
			MouseEventKind::Down(_) => {
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
				
				
				if let Some(byte_column) = byte_column &&
					mouse_event.row as usize - tab_bar_rows < self.window_size.hex_rows()
				{
					current_buffer.primary_cursor = Cursor::at(
						current_buffer.scroll_position +
						(mouse_event.row as usize - tab_bar_rows) * BYTES_PER_LINE +
						byte_column
					);
					current_buffer.cursors.clear();
				}
			},
			MouseEventKind::ScrollDown => {
				for _ in 0..3 {
					current_buffer.scroll_down(self.window_size);
				}
			},
			MouseEventKind::ScrollUp => {
				for _ in 0..3 {
					current_buffer.scroll_up(self.window_size);
				}
			},
			_ => (),
		}
	}
	
	fn quit_if_saved(&mut self) {
		if self.buffers.iter().all(Buffer::all_changes_saved) {
			self.quit();
		} else {
			self.buffers[self.current_buffer_index].alert_message = Span::from(
				"there are unsaved changes, use Q to override"
			).red();
		}
	}
	
	const fn quit(&mut self) {
		self.should_quit = true;
	}
	
	const fn previous_buffer(&mut self) {
		if self.current_buffer_index == 0 {
			self.current_buffer_index = self.buffers.len() - 1;
		} else {
			self.current_buffer_index -= 1;
		}
	}
	
	const fn next_buffer(&mut self) {
		if self.current_buffer_index == self.buffers.len() - 1 {
			self.current_buffer_index = 0;
		} else {
			self.current_buffer_index += 1;
		}
	}
	
	fn yank(&mut self) {
		let current_buffer = &self.buffers[self.current_buffer_index];
		
		self.primary_cursor_register = current_buffer
			.contents[current_buffer.primary_cursor.range()]
			.to_vec();
		
		self.other_cursor_registers = current_buffer.cursors
			.iter()
			.map(|cursor| {
				current_buffer.contents[cursor.range()].to_vec()
			})
			.collect();
	}
}

impl WindowSize {
	pub const fn visible_byte_count(&self) -> usize {
		self.hex_rows() * BYTES_PER_LINE
	}
	
	pub const fn hex_rows(&self) -> usize {
		self.rows - self.covered_rows
	}
}
