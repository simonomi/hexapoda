use std::{env, process::exit};
use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyModifiers}, terminal::window_size};
use ratatui::{style::Stylize, text::Span};
use crate::{BYTES_PER_LINE, action::AppAction, buffer::Buffer, config::Config};

mod widget;

pub struct App {
	pub config: Config,
	
	pub buffers: Vec<Buffer>,
	pub current_buffer_index: usize,
	
	pub window_size: WindowSize,
	
	pub should_quit: bool,
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
		
		Self {
			config: Config::default(),
			
			buffers,
			current_buffer_index: 0,
			
			window_size: WindowSize {
				rows: window_size().unwrap().rows as usize,
				// 1 because of the status line
				covered_rows: 1,
			},
			
			should_quit: false,
		}
	}
	
	#[allow(clippy::too_many_lines)]
	pub fn handle_events(&mut self) {
		#[allow(clippy::collapsible_match)]
		match event::read().unwrap() {
			Event::Resize(_, height) => {
				self.window_size.rows = height as usize;
			}
			Event::Key(key_event) => self.handle_key(key_event),
			// Event::Mouse(mouse_event) => {
			// 	mouse_event.kind
			// },
			_ => {}
		}
	}
	
	fn handle_key(&mut self, key_event: KeyEvent) {
		if key_event.modifiers == KeyModifiers::CONTROL &&
		   key_event.code == KeyCode::Char('c')
		{
			crossterm::terminal::disable_raw_mode().unwrap();
			ratatui::restore();
			exit(130);
		}
		
		let maybe_app_action = self.buffers[self.current_buffer_index].handle_key(
			key_event,
			&self.config,
			self.window_size
		);
		
		if let Some(app_action) = maybe_app_action {
			match app_action {
				AppAction::QuitIfSaved => self.quit_if_saved(),
				AppAction::Quit => self.quit(),
				
				AppAction::PreviousBuffer => self.previous_buffer(),
				AppAction::NextBuffer => self.next_buffer(),
			}
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
	
	pub fn current_buffer(&self) -> &Buffer {
		&self.buffers[self.current_buffer_index]
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
