use std::{cmp::min, mem::swap};

use crate::{BYTES_PER_LINE, app::{App, Mode, PartialAction}};

#[derive(Clone, Copy)]
pub enum Action {
	Quit,
	
	NormalMode,
	SelectMode,
	
	GPartial,
	ZPartial,
	RPartial,
	
	MoveByteUp,
	MoveByteDown,
	MoveByteLeft,
	MoveByteRight,
	
	GotoLineStart,
	GotoLineEnd,
	GotoFileStart,
	GotoFileEnd,
	
	ScrollDown,
	ScrollUp,
	
	PageCursorHalfDown,
	PageCursorHalfUp,
	
	PageDown,
	PageUp,
	
	MoveNextWordStart,
	MoveNextWordEnd,
	MovePreviousWordStart,
	
	CollapseSelection,
	
	ExtendLineBelow,
	ExtendLineAbove,
}

impl App {
	pub fn execute(&mut self, action: Action) {
		match action {
			Action::Quit => self.quit(),
			
			Action::NormalMode => self.normal_mode(),
			Action::SelectMode => self.select_mode(),
			
			Action::GPartial => self.g_partial(),
			Action::ZPartial => self.z_partial(),
			Action::RPartial => self.r_partial(),
			
			Action::MoveByteUp => self.move_byte_up(),
			Action::MoveByteDown => self.move_byte_down(),
			Action::MoveByteLeft => self.move_byte_left(),
			Action::MoveByteRight => self.move_byte_right(),
			
			Action::GotoLineStart => self.goto_line_start(),
			Action::GotoLineEnd => self.goto_line_end(),
			Action::GotoFileStart => self.goto_file_start(),
			Action::GotoFileEnd => self.goto_file_end(),
			
			Action::ScrollDown => self.scroll_down(),
			Action::ScrollUp => self.scroll_up(),
			
			Action::PageCursorHalfDown => self.page_cursor_half_down(),
			Action::PageCursorHalfUp => self.page_cursor_half_up(),
			
			Action::PageDown => self.page_down(),
			Action::PageUp => self.page_up(),
			
			Action::MoveNextWordStart => self.move_next_word_start(),
			Action::MoveNextWordEnd => self.move_next_word_end(),
			Action::MovePreviousWordStart => self.move_previous_word_start(),
			
			Action::CollapseSelection => self.collapse_selection(),
			
			Action::ExtendLineBelow => self.extend_line_below(),
			Action::ExtendLineAbove => self.extend_line_above(),
		}
	}
	
	const fn quit(&mut self) {
		self.should_quit = true;
	}
	
	const fn normal_mode(&mut self) {
		self.mode = Mode::Normal;
	}
	
	const fn select_mode(&mut self) {
		self.mode = Mode::Select;
	}
	
	const fn g_partial(&mut self) {
		self.partial_action = Some(PartialAction::Goto);
	}
	
	const fn z_partial(&mut self) {
		self.partial_action = Some(PartialAction::Zview);
	}
	
	const fn r_partial(&mut self) {
		self.partial_action = Some(PartialAction::Replace);
	}
	
	const fn move_byte_up(&mut self) {
		if self.cursor.head >= BYTES_PER_LINE {
			self.cursor.head -= BYTES_PER_LINE;
			self.cursor.collapse();
			
			self.clamp_screen_to_cursor();
		}
	}
	
	const fn move_byte_down(&mut self) {
		if self.contents.len() - 1 - self.cursor.head >= BYTES_PER_LINE {
			self.cursor.head += BYTES_PER_LINE;
			self.cursor.collapse();
			
			self.clamp_screen_to_cursor();
		}
	}
	
	const fn move_byte_left(&mut self) {
		if self.cursor.head >= 1 {
			self.cursor.head -= 1;
			self.cursor.collapse();
			
			self.clamp_screen_to_cursor();
		}
	}
	
	const fn move_byte_right(&mut self) {
		if self.contents.len() - 1 - self.cursor.head >= 1 {
			self.cursor.head += 1;
			self.cursor.collapse();
			
			self.clamp_screen_to_cursor();
		}
	}
	
	const fn goto_line_start(&mut self) {
		self.cursor.head -= self.cursor.head % BYTES_PER_LINE;
		self.cursor.collapse();
	}
	
	const fn goto_line_end(&mut self) {
		self.cursor.head += BYTES_PER_LINE - 1 - (self.cursor.head % BYTES_PER_LINE);
		self.cursor.collapse();
	}
	
	const fn goto_file_start(&mut self) {
		self.cursor.head %= BYTES_PER_LINE;
		self.cursor.collapse();
		self.clamp_screen_to_cursor();
	}
	
	const fn goto_file_end(&mut self) {
		self.cursor.head = previous_multiple_of(BYTES_PER_LINE, self.contents.len()) +
			(self.cursor.head % BYTES_PER_LINE);
		
		self.cursor.collapse();
		self.clamp_screen_to_cursor();
	}
	
	fn scroll_down(&mut self) {
		self.scroll_position = min(
			self.scroll_position + BYTES_PER_LINE,
			self.contents.len() - (5 * BYTES_PER_LINE)
		);
		self.cursor.clamp(self.scroll_position, self.screen_size());
	}
	
	fn scroll_up(&mut self) {
		self.scroll_position = self.scroll_position.saturating_sub(BYTES_PER_LINE);
		self.cursor.clamp(self.scroll_position, self.screen_size());
	}
	
	fn page_cursor_half_down(&mut self) {
		let head_offset = self.cursor.head - self.scroll_position;
		let tail_offset = self.cursor.tail - self.scroll_position;
		
		self.scroll_position = min(
			self.scroll_position + (self.screen_size() / 2).next_multiple_of(BYTES_PER_LINE),
			self.contents.len() - (5 * BYTES_PER_LINE)
		);
		
		self.cursor.head = (self.scroll_position + head_offset).min(self.contents.len() - 1);
		self.cursor.tail = (self.scroll_position + tail_offset).min(self.contents.len() - 1);
	}
	
	fn page_cursor_half_up(&mut self) {
		let head_offset = self.cursor.head - self.scroll_position;
		let tail_offset = self.cursor.tail - self.scroll_position;
		
		self.scroll_position = self.scroll_position.saturating_sub(
			(self.screen_size() / 2).next_multiple_of(BYTES_PER_LINE)
		);
		
		self.cursor.head = (self.scroll_position + head_offset).min(self.contents.len() - 1);
		self.cursor.tail = (self.scroll_position + tail_offset).min(self.contents.len() - 1);
	}
	
	fn page_down(&mut self) {
		self.scroll_position = min(
			self.scroll_position + self.screen_size(),
			self.contents.len() - (5 * BYTES_PER_LINE)
		);
		self.cursor.clamp(self.scroll_position, self.screen_size());
	}
	
	fn page_up(&mut self) {
		self.scroll_position = self.scroll_position.saturating_sub(
			self.screen_size()
		);
		self.cursor.clamp(self.scroll_position, self.screen_size());
	}
	
	fn move_next_word_start(&mut self) {
		self.cursor.move_to_next_word(self.contents.len() - 1);
		self.clamp_screen_to_cursor();
	}
	
	fn move_next_word_end(&mut self) {
		self.cursor.move_to_next_end(self.contents.len() - 1);
		self.clamp_screen_to_cursor();
	}
	
	const fn move_previous_word_start(&mut self) {
		self.cursor.move_to_previous_beginning();
		self.clamp_screen_to_cursor();
	}
	
	const fn collapse_selection(&mut self) {
		self.cursor.collapse();
	}
	
	fn extend_line_below(&mut self) {
		if self.cursor.tail > self.cursor.head {
			swap(&mut self.cursor.head, &mut self.cursor.tail);
		}
		
		if self.cursor.tail.is_multiple_of(BYTES_PER_LINE) &&
           self.cursor.head % BYTES_PER_LINE == BYTES_PER_LINE - 1
		{
			self.cursor.head = min(
				self.cursor.head + BYTES_PER_LINE,
				self.contents.len() - 1
			);
		} else {
			self.cursor.tail -= self.cursor.tail % BYTES_PER_LINE;
			self.cursor.head += BYTES_PER_LINE - 1 - (self.cursor.head % BYTES_PER_LINE);
		}
	}
	
	const fn extend_line_above(&mut self) {
		if self.cursor.head > self.cursor.tail {
			swap(&mut self.cursor.head, &mut self.cursor.tail);
		}
		
		if self.cursor.head.is_multiple_of(BYTES_PER_LINE) &&
		   (self.cursor.tail % BYTES_PER_LINE == BYTES_PER_LINE - 1 ||
		    self.cursor.tail == self.contents.len() - 1)
		{
			self.cursor.head = self.cursor.head.saturating_sub(BYTES_PER_LINE);
		} else {
			self.cursor.head -= self.cursor.head % BYTES_PER_LINE;
			self.cursor.tail += BYTES_PER_LINE - 1 - (self.cursor.tail % BYTES_PER_LINE);
		}
	}
}

// helpers
impl App {
	// in bytes
	const fn screen_size(&self) -> usize {
		self.window_rows * BYTES_PER_LINE
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
