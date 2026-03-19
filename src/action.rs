use std::{cmp::min, fs::File, io::Write, mem::{replace, swap}};
use crate::{BYTES_PER_LINE, app::WindowSize, buffer::{Buffer, Mode, PartialAction}, edit_action::EditAction};

#[derive(Clone, Copy)]
pub enum Action {
	QuitIfSaved,
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
	
	PreviousBuffer,
	NextBuffer,
}

// actions that act on the app as a whole, not just one buffer
pub enum AppAction {
	QuitIfSaved,
	Quit,
	
	PreviousBuffer,
	NextBuffer,
}

impl Buffer {
	pub fn execute(&mut self, action: Action, window_size: WindowSize) -> Option<AppAction> {
		match action {
			Action::QuitIfSaved => return Some(AppAction::QuitIfSaved),
			Action::Quit => return Some(AppAction::Quit),
			
			Action::NormalMode => self.normal_mode(),
			Action::SelectMode => self.select_mode(),
			
			Action::Goto => self.goto(),
			Action::View => self.view(),
			Action::Replace => self.replace(),
			Action::Space => self.space(),
			
			Action::MoveByteUp => self.move_byte_up(window_size),
			Action::MoveByteDown => self.move_byte_down(window_size),
			Action::MoveByteLeft => self.move_byte_left(window_size),
			Action::MoveByteRight => self.move_byte_right(window_size),
			
			Action::ExtendByteUp => self.extend_byte_up(window_size),
			Action::ExtendByteDown => self.extend_byte_down(window_size),
			Action::ExtendByteLeft => self.extend_byte_left(window_size),
			Action::ExtendByteRight => self.extend_byte_right(window_size),
			
			Action::GotoLineStart => self.goto_line_start(),
			Action::GotoLineEnd => self.goto_line_end(),
			Action::GotoFileStart => self.goto_file_start(window_size),
			Action::GotoFileEnd => self.goto_file_end(window_size),
			
			Action::ScrollDown => self.scroll_down(window_size),
			Action::ScrollUp => self.scroll_up(window_size),
			
			Action::PageCursorHalfDown => self.page_cursor_half_down(window_size),
			Action::PageCursorHalfUp => self.page_cursor_half_up(window_size),
			
			Action::PageDown => self.page_down(window_size),
			Action::PageUp => self.page_up(window_size),
			
			Action::MoveNextWordStart => self.move_next_word_start(window_size),
			Action::MoveNextWordEnd => self.move_next_word_end(window_size),
			Action::MovePreviousWordStart => self.move_previous_word_start(window_size),
			
			Action::ExtendNextWordStart => self.extend_next_word_start(window_size),
			Action::ExtendNextWordEnd => self.extend_next_word_end(window_size),
			Action::ExtendPreviousWordStart => self.extend_previous_word_start(window_size),
			
			Action::CollapseSelection => self.collapse_selection(),
			
			Action::ExtendLineBelow => self.extend_line_below(),
			Action::ExtendLineAbove => self.extend_line_above(),
			
			Action::Delete => self.delete(),
			
			Action::Undo => self.undo(),
			Action::Redo => self.redo(),
			
			Action::Save => self.save(),
			
			Action::PreviousBuffer => return Some(AppAction::PreviousBuffer),
			Action::NextBuffer => return Some(AppAction::NextBuffer),
		}
		
		None
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
	
	const fn move_byte_up(&mut self, window_size: WindowSize) {
		if self.cursor.head >= BYTES_PER_LINE {
			self.cursor.head -= BYTES_PER_LINE;
			self.cursor.collapse();
			
			self.clamp_screen_to_cursor(window_size);
		}
	}
	
	const fn move_byte_down(&mut self, window_size: WindowSize) {
		if self.max_contents_index() - self.cursor.head >= BYTES_PER_LINE {
			self.cursor.head += BYTES_PER_LINE;
			self.cursor.collapse();
			
			self.clamp_screen_to_cursor(window_size);
		}
	}
	
	const fn move_byte_left(&mut self, window_size: WindowSize) {
		if self.cursor.head >= 1 {
			self.cursor.head -= 1;
			self.cursor.collapse();
			
			self.clamp_screen_to_cursor(window_size);
		}
	}
	
	const fn move_byte_right(&mut self, window_size: WindowSize) {
		if self.max_contents_index() - self.cursor.head >= 1 {
			self.cursor.head += 1;
			self.cursor.collapse();
			
			self.clamp_screen_to_cursor(window_size);
		}
	}
	
	const fn extend_byte_up(&mut self, window_size: WindowSize) {
		if self.cursor.head >= BYTES_PER_LINE {
			self.cursor.head -= BYTES_PER_LINE;
			self.clamp_screen_to_cursor(window_size);
		}
	}
	
	const fn extend_byte_down(&mut self, window_size: WindowSize) {
		if self.max_contents_index() - self.cursor.head >= BYTES_PER_LINE {
			self.cursor.head += BYTES_PER_LINE;
			self.clamp_screen_to_cursor(window_size);
		}
	}
	
	const fn extend_byte_left(&mut self, window_size: WindowSize) {
		if self.cursor.head >= 1 {
			self.cursor.head -= 1;
			self.clamp_screen_to_cursor(window_size);
		}
	}
	
	const fn extend_byte_right(&mut self, window_size: WindowSize) {
		if self.max_contents_index() - self.cursor.head >= 1 {
			self.cursor.head += 1;
			self.clamp_screen_to_cursor(window_size);
		}
	}
	
	const fn goto_line_start(&mut self) {
		self.cursor.head -= self.cursor.head % BYTES_PER_LINE;
		self.cursor.collapse();
	}
	
	fn goto_line_end(&mut self) {
		self.cursor.head = min(
			self.cursor.head + BYTES_PER_LINE - 1 - (self.cursor.head % BYTES_PER_LINE),
			self.max_contents_index()
		);
		self.cursor.collapse();
	}
	
	const fn goto_file_start(&mut self, window_size: WindowSize) {
		self.cursor.head %= BYTES_PER_LINE;
		self.cursor.collapse();
		self.clamp_screen_to_cursor(window_size);
	}
	
	const fn goto_file_end(&mut self, window_size: WindowSize) {
		self.cursor.head = previous_multiple_of(BYTES_PER_LINE, self.contents.len()) +
			(self.cursor.head % BYTES_PER_LINE);
		
		self.cursor.collapse();
		self.clamp_screen_to_cursor(window_size);
	}
	
	fn scroll_down(&mut self, window_size: WindowSize) {
		if self.contents.len() <= 5 * BYTES_PER_LINE { return; }
		
		self.scroll_position = min(
			self.scroll_position + BYTES_PER_LINE,
			self.contents.len() - (5 * BYTES_PER_LINE)
		);
		self.cursor.clamp(self.scroll_position, window_size.visible_byte_count());
	}
	
	fn scroll_up(&mut self, window_size: WindowSize) {
		self.scroll_position = self.scroll_position.saturating_sub(BYTES_PER_LINE);
		self.cursor.clamp(self.scroll_position, window_size.visible_byte_count());
	}
	
	fn page_cursor_half_down(&mut self, window_size: WindowSize) {
		if self.contents.len() <= 5 * BYTES_PER_LINE { return; }
		
		let head_offset = self.cursor.head - self.scroll_position;
		let tail_offset = self.cursor.tail - self.scroll_position;
		
		self.scroll_position = min(
			self.scroll_position + (window_size.visible_byte_count() / 2).next_multiple_of(BYTES_PER_LINE),
			self.contents.len() - (5 * BYTES_PER_LINE)
		);
		
		self.cursor.head = (self.scroll_position + head_offset).min(self.max_contents_index());
		self.cursor.tail = (self.scroll_position + tail_offset).min(self.max_contents_index());
	}
	
	fn page_cursor_half_up(&mut self, window_size: WindowSize) {
		let head_offset = self.cursor.head - self.scroll_position;
		let tail_offset = self.cursor.tail - self.scroll_position;
		
		self.scroll_position = self.scroll_position.saturating_sub(
			(window_size.visible_byte_count() / 2).next_multiple_of(BYTES_PER_LINE)
		);
		
		self.cursor.head = (self.scroll_position + head_offset).min(self.max_contents_index());
		self.cursor.tail = (self.scroll_position + tail_offset).min(self.max_contents_index());
	}
	
	fn page_down(&mut self, window_size: WindowSize) {
		if self.contents.len() <= 5 * BYTES_PER_LINE { return; }
		
		self.scroll_position = min(
			self.scroll_position + window_size.visible_byte_count(),
			self.contents.len() - (5 * BYTES_PER_LINE)
		);
		self.cursor.clamp(self.scroll_position, window_size.visible_byte_count());
	}
	
	fn page_up(&mut self, window_size: WindowSize) {
		self.scroll_position = self.scroll_position.saturating_sub(
			window_size.visible_byte_count()
		);
		self.cursor.clamp(self.scroll_position, window_size.visible_byte_count());
	}
	
	fn move_next_word_start(&mut self, window_size: WindowSize) {
		self.cursor.move_to_next_word(self.max_contents_index());
		self.clamp_screen_to_cursor(window_size);
	}
	
	fn move_next_word_end(&mut self, window_size: WindowSize) {
		self.cursor.move_to_next_end(self.max_contents_index());
		self.clamp_screen_to_cursor(window_size);
	}
	
	const fn move_previous_word_start(&mut self, window_size: WindowSize) {
		self.cursor.move_to_previous_beginning();
		self.clamp_screen_to_cursor(window_size);
	}
	
	fn extend_next_word_start(&mut self, window_size: WindowSize) {
		self.cursor.extend_to_next_word(self.max_contents_index());
		self.clamp_screen_to_cursor(window_size);
	}
	
	fn extend_next_word_end(&mut self, window_size: WindowSize) {
		self.cursor.extend_to_next_end(self.max_contents_index());
		self.clamp_screen_to_cursor(window_size);
	}
	
	const fn extend_previous_word_start(&mut self, window_size: WindowSize) {
		self.cursor.extend_to_previous_beginning();
		self.clamp_screen_to_cursor(window_size);
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
impl Buffer {
	const fn clamp_screen_to_cursor(&mut self, window_size: WindowSize) {
		if self.cursor.head < self.scroll_position {
			self.scroll_position -= (self.scroll_position - self.cursor.head).next_multiple_of(BYTES_PER_LINE);
		} else if self.cursor.head > self.scroll_position + window_size.visible_byte_count() - 1 {
			let screen_edge_offset_to_cursor = self.cursor.head - (self.scroll_position + window_size.visible_byte_count() - 1);
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
