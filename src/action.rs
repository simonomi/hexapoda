use std::{cmp::min, fs::File, io::Write, mem::{replace, swap}};

use crate::{BYTES_PER_LINE, app::{App, Mode, PartialAction}, edit_action::EditAction};

#[derive(Clone, Copy)]
pub enum Action {
	Quit,
	
	NormalMode,
	SelectMode,
	
	Goto,
	View,
	Replace,
	Space,
	
	MoveByteUp,
	MoveByteDown,
	MoveByteLeft,
	MoveByteRight,
	
	ExtendByteUp,
	ExtendByteDown,
	ExtendByteLeft,
	ExtendByteRight,
	
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
	
	ExtendNextWordStart,
	ExtendNextWordEnd,
	ExtendPreviousWordStart,
	
	CollapseSelection,
	
	ExtendLineBelow,
	ExtendLineAbove,
	
	Delete,
	
	Undo,
	Redo,
	
	Save,
}

impl App {
	pub fn execute(&mut self, action: Action) {
		match action {
			Action::Quit => self.quit(),
			
			Action::NormalMode => self.normal_mode(),
			Action::SelectMode => self.select_mode(),
			
			Action::Goto => self.goto(),
			Action::View => self.view(),
			Action::Replace => self.replace(),
			Action::Space => self.space(),
			
			Action::MoveByteUp => self.move_byte_up(),
			Action::MoveByteDown => self.move_byte_down(),
			Action::MoveByteLeft => self.move_byte_left(),
			Action::MoveByteRight => self.move_byte_right(),
			
			Action::ExtendByteUp => self.extend_byte_up(),
			Action::ExtendByteDown => self.extend_byte_down(),
			Action::ExtendByteLeft => self.extend_byte_left(),
			Action::ExtendByteRight => self.extend_byte_right(),
			
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
			
			Action::ExtendNextWordStart => self.extend_next_word_start(),
			Action::ExtendNextWordEnd => self.extend_next_word_end(),
			Action::ExtendPreviousWordStart => self.extend_previous_word_start(),
			
			Action::CollapseSelection => self.collapse_selection(),
			
			Action::ExtendLineBelow => self.extend_line_below(),
			Action::ExtendLineAbove => self.extend_line_above(),
			
			Action::Delete => self.delete(),
			
			Action::Undo => self.undo(),
			Action::Redo => self.redo(),
			
			Action::Save => self.save(),
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
	
	const fn goto(&mut self) {
		self.partial_action = Some(PartialAction::Goto);
	}
	
	const fn view(&mut self) {
		self.partial_action = Some(PartialAction::View);
	}
	
	const fn replace(&mut self) {
		if !self.contents.is_empty() {
			self.partial_action = Some(PartialAction::Replace);
		}
	}
	
	const fn space(&mut self) {
		self.partial_action = Some(PartialAction::Space);
	}
	
	const fn move_byte_up(&mut self) {
		if self.cursor.head >= BYTES_PER_LINE {
			self.cursor.head -= BYTES_PER_LINE;
			self.cursor.collapse();
			
			self.clamp_screen_to_cursor();
		}
	}
	
	const fn move_byte_down(&mut self) {
		if self.max_contents_index() - self.cursor.head >= BYTES_PER_LINE {
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
		if self.max_contents_index() - self.cursor.head >= 1 {
			self.cursor.head += 1;
			self.cursor.collapse();
			
			self.clamp_screen_to_cursor();
		}
	}
	
	const fn extend_byte_up(&mut self) {
		if self.cursor.head >= BYTES_PER_LINE {
			self.cursor.head -= BYTES_PER_LINE;
			self.clamp_screen_to_cursor();
		}
	}
	
	const fn extend_byte_down(&mut self) {
		if self.max_contents_index() - self.cursor.head >= BYTES_PER_LINE {
			self.cursor.head += BYTES_PER_LINE;
			self.clamp_screen_to_cursor();
		}
	}
	
	const fn extend_byte_left(&mut self) {
		if self.cursor.head >= 1 {
			self.cursor.head -= 1;
			self.clamp_screen_to_cursor();
		}
	}
	
	const fn extend_byte_right(&mut self) {
		if self.max_contents_index() - self.cursor.head >= 1 {
			self.cursor.head += 1;
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
		if self.contents.len() <= 5 * BYTES_PER_LINE { return; }
		
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
		if self.contents.len() <= 5 * BYTES_PER_LINE { return; }
		
		let head_offset = self.cursor.head - self.scroll_position;
		let tail_offset = self.cursor.tail - self.scroll_position;
		
		self.scroll_position = min(
			self.scroll_position + (self.screen_size() / 2).next_multiple_of(BYTES_PER_LINE),
			self.contents.len() - (5 * BYTES_PER_LINE)
		);
		
		self.cursor.head = (self.scroll_position + head_offset).min(self.max_contents_index());
		self.cursor.tail = (self.scroll_position + tail_offset).min(self.max_contents_index());
	}
	
	fn page_cursor_half_up(&mut self) {
		let head_offset = self.cursor.head - self.scroll_position;
		let tail_offset = self.cursor.tail - self.scroll_position;
		
		self.scroll_position = self.scroll_position.saturating_sub(
			(self.screen_size() / 2).next_multiple_of(BYTES_PER_LINE)
		);
		
		self.cursor.head = (self.scroll_position + head_offset).min(self.max_contents_index());
		self.cursor.tail = (self.scroll_position + tail_offset).min(self.max_contents_index());
	}
	
	fn page_down(&mut self) {
		if self.contents.len() <= 5 * BYTES_PER_LINE { return; }
		
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
		self.cursor.move_to_next_word(self.max_contents_index());
		self.clamp_screen_to_cursor();
	}
	
	fn move_next_word_end(&mut self) {
		self.cursor.move_to_next_end(self.max_contents_index());
		self.clamp_screen_to_cursor();
	}
	
	const fn move_previous_word_start(&mut self) {
		self.cursor.move_to_previous_beginning();
		self.clamp_screen_to_cursor();
	}
	
	fn extend_next_word_start(&mut self) {
		self.cursor.extend_to_next_word(self.max_contents_index());
		self.clamp_screen_to_cursor();
	}
	
	fn extend_next_word_end(&mut self) {
		self.cursor.extend_to_next_end(self.max_contents_index());
		self.clamp_screen_to_cursor();
	}
	
	const fn extend_previous_word_start(&mut self) {
		self.cursor.extend_to_previous_beginning();
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
				self.max_contents_index()
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
		    self.cursor.tail == self.max_contents_index())
		{
			self.cursor.head = self.cursor.head.saturating_sub(BYTES_PER_LINE);
		} else {
			self.cursor.head -= self.cursor.head % BYTES_PER_LINE;
			self.cursor.tail += BYTES_PER_LINE - 1 - (self.cursor.tail % BYTES_PER_LINE);
		}
	}
	
	fn delete(&mut self) {
		if !self.contents.is_empty() {
			self.execute_and_add(
				EditAction::Delete {
					cursor: self.cursor,
					old_data: self.contents[self.cursor.range()].into()
				}
			);
		}
		
		if self.mode == Mode::Select {
			self.mode = Mode::Normal;
		}
	}
	
	fn undo(&mut self) {
		if self.time_traveling == Some(0) || self.edit_history.is_empty() { return; }
		
		let current_date = self.time_traveling
			.map_or(self.edit_history.len() - 1, |date| date - 1);
		
		self.time_traveling = Some(current_date);
		
		let edit_action = replace(
			&mut self.edit_history[current_date],
			EditAction::Placeholder
		);
		
		self.undo_edit(&edit_action);
		
		self.edit_history[current_date] = edit_action;
	}
	
	fn redo(&mut self) {
		let Some(previous_date) = self.time_traveling else { return; };
		
		let current_date = previous_date + 1;
		
		self.time_traveling = if current_date == self.edit_history.len() {
			None
		} else {
			Some(current_date)
		};
		
		let edit_action = replace(
			&mut self.edit_history[previous_date],
			EditAction::Placeholder
		);
		
		self.execute_edit(&edit_action);
		
		self.edit_history[previous_date] = edit_action;
	}
	
	fn save(&mut self) {
		let mut file = File::create(&self.file_path).unwrap();
		file.write_all(&self.contents).unwrap();
		
		self.last_saved_at = Some(
			self.time_traveling.unwrap_or(self.edit_history.len())
		);
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
