use ratatui::{style::Stylize, text::Span};

use crate::{app::App, buffer::Buffer};

impl App {
	pub fn quit_if_saved(&mut self) {
		if self.buffers.iter().all(Buffer::all_changes_saved) {
			self.quit();
		} else {
			self.buffers[self.current_buffer_index].alert_message = Span::from(
				"there are unsaved changes, use Q to override"
			).red();
		}
	}
	
	pub const fn quit(&mut self) {
		self.should_quit = true;
	}
	
	pub const fn previous_buffer(&mut self) {
		if self.current_buffer_index == 0 {
			self.current_buffer_index = self.buffers.len() - 1;
		} else {
			self.current_buffer_index -= 1;
		}
	}
	
	pub const fn next_buffer(&mut self) {
		if self.current_buffer_index == self.buffers.len() - 1 {
			self.current_buffer_index = 0;
		} else {
			self.current_buffer_index += 1;
		}
	}
	
	pub fn yank(&mut self) {
		let current_buffer = &mut self.buffers[self.current_buffer_index];
		
		self.primary_cursor_register = current_buffer
			.contents[current_buffer.primary_cursor.range()]
			.to_vec();
		
		self.other_cursor_registers = current_buffer.cursors
			.iter()
			.map(|cursor| {
				current_buffer.contents[cursor.range()].to_vec()
			})
			.collect();
		
		current_buffer.alert_message = if current_buffer.cursors.is_empty() {
			"yanked 1 selection".into()
		} else {
			format!("yanked {} selections", current_buffer.cursors.len() + 1).into()
		};
	}
}
