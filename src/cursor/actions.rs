use std::{cmp::min, mem::swap};
use crate::{BYTES_PER_LINE, action::CursorAction, cursor::Cursor};

impl Cursor {
	pub fn execute(
		&mut self,
		action: CursorAction,
		max_contents_index: usize
	) {
		match action {
			CursorAction::MoveByteUp => self.move_byte_up(),
			CursorAction::MoveByteDown => self.move_byte_down(max_contents_index),
			CursorAction::MoveByteLeft => self.move_byte_left(),
			CursorAction::MoveByteRight => self.move_byte_right(max_contents_index),
			
			CursorAction::ExtendByteUp => self.extend_byte_up(),
			CursorAction::ExtendByteDown => self.extend_byte_down(max_contents_index),
			CursorAction::ExtendByteLeft => self.extend_byte_left(),
			CursorAction::ExtendByteRight => self.extend_byte_right(max_contents_index),
			
			CursorAction::GotoLineStart => self.goto_line_start(),
			CursorAction::GotoLineEnd => self.goto_line_end(max_contents_index),
			CursorAction::GotoFileStart => self.goto_file_start(),
			CursorAction::GotoFileEnd => self.goto_file_end(max_contents_index),
			
			CursorAction::ExtendLineStart => self.extend_line_start(),
			CursorAction::ExtendLineEnd => self.extend_line_end(max_contents_index),
			CursorAction::ExtendFileStart => self.extend_file_start(),
			CursorAction::ExtendFileEnd => self.extend_file_end(max_contents_index),
			
			CursorAction::MoveNextWordStart => self.move_next_word_start(max_contents_index),
			CursorAction::MoveNextWordEnd => self.move_next_word_end(max_contents_index),
			CursorAction::MovePreviousWordStart => self.move_previous_word_start(),
			
			CursorAction::ExtendNextWordStart => self.extend_next_word_start(max_contents_index),
			CursorAction::ExtendNextWordEnd => self.extend_next_word_end(max_contents_index),
			CursorAction::ExtendPreviousWordStart => self.extend_previous_word_start(),
			
			CursorAction::ExtendLineBelow => self.extend_line_below(max_contents_index),
			CursorAction::ExtendLineAbove => self.extend_line_above(max_contents_index),
		}
	}
	
	pub const fn move_byte_up(&mut self) {
		if self.head >= BYTES_PER_LINE {
			self.head -= BYTES_PER_LINE;
			self.collapse();
		}
	}
	
	pub const fn move_byte_down(&mut self, max: usize) {
		if max - self.head >= BYTES_PER_LINE {
			self.head += BYTES_PER_LINE;
			self.collapse();
		}
	}
	
	pub const fn move_byte_left(&mut self) {
		if self.head >= 1 {
			self.head -= 1;
			self.collapse();
		}
	}
	
	pub const fn move_byte_right(&mut self, max: usize) {
		if max - self.head >= 1 {
			self.head += 1;
			self.collapse();
		}
	}
	
	pub const fn extend_byte_up(&mut self) {
		if self.head >= BYTES_PER_LINE {
			self.head -= BYTES_PER_LINE;
		}
	}
	
	pub const fn extend_byte_down(&mut self, max: usize) {
		if max - self.head >= BYTES_PER_LINE {
			self.head += BYTES_PER_LINE;
		}
	}
	
	pub const fn extend_byte_left(&mut self) {
		if self.head >= 1 {
			self.head -= 1;
		}
	}
	
	pub const fn extend_byte_right(&mut self, max: usize) {
		if max - self.head >= 1 {
			self.head += 1;
		}
	}
	
	pub const fn goto_line_start(&mut self) {
		self.extend_line_start();
		self.collapse();
	}
	
	pub fn goto_line_end(&mut self, max: usize) {
		self.extend_line_end(max);
		self.collapse();
	}
	
	pub const fn goto_file_start(&mut self) {
		self.extend_file_start();
		self.collapse();
	}
	
	pub const fn goto_file_end(&mut self, max: usize) {
		self.extend_file_end(max);
		self.collapse();
	}
	
	pub const fn extend_line_start(&mut self) {
		self.head -= self.head % BYTES_PER_LINE;
	}
	
	pub fn extend_line_end(&mut self, max: usize) {
		self.head = min(
			self.head + BYTES_PER_LINE - 1 - (self.head % BYTES_PER_LINE),
			max
		);
	}
	
	pub const fn extend_file_start(&mut self) {
		self.head %= BYTES_PER_LINE;
	}
	
	pub const fn extend_file_end(&mut self, max: usize) {
		self.head += previous_multiple_of(BYTES_PER_LINE, max + 1 - self.head);
		
	}
	
	pub fn move_next_word_start(&mut self, max: usize) {
		if self.head == max { return; }
		
		if self.head.is_multiple_of(4) { // at the beginning of a word
			self.head = (self.head + 4).min(max);
		} else {
			self.head = self.head.next_multiple_of(4).min(max);
		}
		self.collapse();
	}
	
	pub fn move_next_word_end(&mut self, max: usize) {
		if self.head == max { return; }
		
		self.collapse();
		if self.head % 4 == 3 { // at the end of a word
			self.tail = self.head + 1;
			self.head = (self.head + 4).min(max);
		} else {
			self.head = ((self.head + 1).next_multiple_of(4) - 1).min(max);
		}
	}
	
	pub const fn move_previous_word_start(&mut self) {
		if self.head == 0 { return; }
		
		self.collapse();
		if self.head.is_multiple_of(4) { // at the beginning of a word
			self.tail = self.head - 1;
			self.head -= 4;
		} else {
			self.head -= self.head % 4;
		}
	}
	
	pub fn extend_next_word_start(&mut self, max: usize) {
		if self.head == max { return; }
		
		if self.head.is_multiple_of(4) { // at the beginning of a word
			self.head = (self.head + 4).min(max);
		} else {
			self.head = self.head.next_multiple_of(4).min(max);
		}
	}
	
	pub fn extend_next_word_end(&mut self, max: usize) {
		if self.head == max { return; }
		
		if self.head % 4 == 3 { // at the end of a word
			self.head = (self.head + 4).min(max);
		} else {
			self.head = ((self.head + 1).next_multiple_of(4) - 1).min(max);
		}
	}
	
	pub const fn extend_previous_word_start(&mut self) {
		if self.head == 0 { return; }
		
		if self.head.is_multiple_of(4) { // at the beginning of a word
			self.head -= 4;
		} else {
			self.head -= self.head % 4;
		}
	}
	
	pub fn extend_line_below(&mut self, max: usize) {
		if self.tail > self.head {
			swap(&mut self.head, &mut self.tail);
		}
		
		if self.tail.is_multiple_of(BYTES_PER_LINE) &&
		   self.head % BYTES_PER_LINE == BYTES_PER_LINE - 1
		{
			self.head = min(self.head + BYTES_PER_LINE, max);
		} else {
			self.tail -= self.tail % BYTES_PER_LINE;
			self.head = min(
				self.head + BYTES_PER_LINE - 1 - (self.head % BYTES_PER_LINE),
				max
			);
		}
	}
	
	pub fn extend_line_above(&mut self, max: usize) {
		if self.head > self.tail {
			swap(&mut self.head, &mut self.tail);
		}
		
		if self.head.is_multiple_of(BYTES_PER_LINE) &&
		   (self.tail % BYTES_PER_LINE == BYTES_PER_LINE - 1 ||
		    self.tail == max)
		{
			self.head = self.head.saturating_sub(BYTES_PER_LINE);
		} else {
			self.head -= self.head % BYTES_PER_LINE;
			self.tail = min(
				self.tail + BYTES_PER_LINE - 1 - (self.tail % BYTES_PER_LINE),
				max
			);
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

mod tests {
	#[allow(unused_imports)]
	use crate::cursor::Cursor;
	
	#[test]
	fn next_word() {
		// [a]bcd efgh -> abcd [e]fgh
		let mut cursor = Cursor::at(0);
		cursor.move_next_word_start(99);
		assert_eq!(cursor, Cursor::at(4));
		
		// a[b]cd efgh -> abcd [e]fgh
		let mut cursor = Cursor::at(1);
		cursor.move_next_word_start(99);
		assert_eq!(cursor, Cursor::at(4));
		
		// ab[c]d efgh -> abcd [e]fgh
		let mut cursor = Cursor::at(2);
		cursor.move_next_word_start(99);
		assert_eq!(cursor, Cursor::at(4));
		
		// abc[d] efgh -> abcd [e]fgh
		let mut cursor = Cursor::at(3);
		cursor.move_next_word_start(99);
		assert_eq!(cursor, Cursor::at(4));
		
		// [a]bcd -> abc[d]
		let mut cursor = Cursor::at(0);
		cursor.move_next_word_start(3);
		assert_eq!(cursor, Cursor::at(3));
		
		// [a]bc -> ab[c]
		let mut cursor = Cursor::at(0);
		cursor.move_next_word_start(2);
		assert_eq!(cursor, Cursor::at(2));
		
		// [a]b -> a[b]
		let mut cursor = Cursor::at(0);
		cursor.move_next_word_start(1);
		assert_eq!(cursor, Cursor::at(1));
		
		// [a] -> [a]
		let mut cursor = Cursor::at(0);
		cursor.move_next_word_start(0);
		assert_eq!(cursor, Cursor::at(0));
		
		// ab[c]d -> abc[d]
		let mut cursor = Cursor::at(2);
		cursor.move_next_word_start(3);
		assert_eq!(cursor, Cursor::at(3));
		
		// abc[d] -> abc[d]
		let mut cursor = Cursor::at(3);
		cursor.move_next_word_start(3);
		assert_eq!(cursor, Cursor::at(3));
		
		// ab[c[d] -> ab[c[d]
		let mut cursor = Cursor { tail: 2, head: 3 };
		cursor.move_next_word_start(3);
		assert_eq!(cursor, Cursor { tail: 2, head: 3 });
	}
	
	#[test]
	fn next_end() {
		// [a]bcd -> [abcd]
		let mut cursor = Cursor::at(0);
		cursor.move_next_word_end(99);
		assert_eq!(cursor, Cursor { tail: 0, head: 3 });
		
		// a[b]cd -> [abcd]
		let mut cursor = Cursor::at(1);
		cursor.move_next_word_end(99);
		assert_eq!(cursor, Cursor { tail: 1, head: 3 });
		
		// ab[c]d -> [abcd]
		let mut cursor = Cursor::at(2);
		cursor.move_next_word_end(99);
		assert_eq!(cursor, Cursor { tail: 2, head: 3 });
		
		// abc[d] efgh -> abcd [efgh]
		let mut cursor = Cursor::at(3);
		cursor.move_next_word_end(99);
		assert_eq!(cursor, Cursor { tail: 4, head: 7 });
		
		// abcd [e]fgh -> abcd [efgh]
		let mut cursor = Cursor::at(4);
		cursor.move_next_word_end(99);
		assert_eq!(cursor, Cursor { tail: 4, head: 7 });
		
		// abcd e[f]gh -> abcd e[fgh]
		let mut cursor = Cursor::at(5);
		cursor.move_next_word_end(99);
		assert_eq!(cursor, Cursor { tail: 5, head: 7 });
		
		// abcd ef[g]h -> abcd ef[gh]
		let mut cursor = Cursor::at(6);
		cursor.move_next_word_end(99);
		assert_eq!(cursor, Cursor { tail: 6, head: 7 });
		
		// abcd efg[h] ijkl -> abcd efgh [ijkl]
		let mut cursor = Cursor::at(7);
		cursor.move_next_word_end(99);
		assert_eq!(cursor, Cursor { tail: 8, head: 11 });
		
		// abcd efg[h] -> abcd efg[h]
		let mut cursor = Cursor::at(7);
		cursor.move_next_word_end(7);
		assert_eq!(cursor, Cursor { tail: 7, head: 7 });
		
		// abcd e[fgh] -> abcd e[fgh]
		let mut cursor = Cursor { tail: 5, head: 7 };
		cursor.move_next_word_end(7);
		assert_eq!(cursor, Cursor { tail: 5, head: 7 });
		
		// a[b]c -> a[bc]
		let mut cursor = Cursor::at(1);
		cursor.move_next_word_end(2);
		assert_eq!(cursor, Cursor { tail: 1, head: 2 });
		
		// a[bc] -> a[bc]
		let mut cursor = Cursor { tail: 1, head: 2};
		cursor.move_next_word_end(2);
		assert_eq!(cursor, Cursor { tail: 1, head: 2 });
		
		// a[b] -> a[b]
		let mut cursor = Cursor::at(1);
		cursor.move_next_word_end(1);
		assert_eq!(cursor, Cursor::at(1));
		
		// [a]b -> [ab]
		let mut cursor = Cursor::at(0);
		cursor.move_next_word_end(1);
		assert_eq!(cursor, Cursor { tail: 0, head: 1 });
		
		// [ab] -> [ab]
		let mut cursor = Cursor { tail: 0, head: 1};
		cursor.move_next_word_end(1);
		assert_eq!(cursor, Cursor { tail: 0, head: 1 });
		
		// [a] -> [a]
		let mut cursor = Cursor::at(0);
		cursor.move_next_word_end(0);
		assert_eq!(cursor, Cursor::at(0));
		
		// [a]bcd] -> [abc[d]
		let mut cursor = Cursor { head: 0, tail: 3 };
		cursor.move_next_word_end(99);
		assert_eq!(cursor, Cursor { tail: 0, head: 3 });
		
		// [a[b]cd -> a[bc[d]
		let mut cursor = Cursor { tail: 0, head: 1 };
		cursor.move_next_word_end(99);
		assert_eq!(cursor, Cursor { tail: 1, head: 3 });
		
		// abc[d] ef -> abcd [ef]
		let mut cursor = Cursor::at(3);
		cursor.move_next_word_end(5);
		assert_eq!(cursor, Cursor { tail: 4, head: 5 });
	}
	
	#[test]
	fn previous_beginning() {
		// abcd efgh [i]jkl -> abcd [efgh] ijkl
		let mut cursor = Cursor::at(8);
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 4, tail: 7 });
		
		// abcd efg[h] -> abcd [efgh]
		let mut cursor = Cursor::at(7);
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 4, tail: 7 });
		
		// abcd ef[g]h -> abcd [efg]h
		let mut cursor = Cursor::at(6);
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 4, tail: 6 });
		
		// abcd e[f]gh -> abcd [ef]gh
		let mut cursor = Cursor::at(5);
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 4, tail: 5 });
		
		// abcd [e]fgh -> [abcd] efgh
		let mut cursor = Cursor::at(4);
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 0, tail: 3 });
		
		// abc[d] -> [abcd]
		let mut cursor = Cursor::at(3);
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 0, tail: 3 });
		
		// ab[c]d -> [abc]d
		let mut cursor = Cursor::at(2);
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 0, tail: 2 });
		
		// a[b]cd -> [ab]cd
		let mut cursor = Cursor::at(1);
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 0, tail: 1 });
		
		// [a]bcd -> [a]bcd
		let mut cursor = Cursor::at(0);
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 0, tail: 0 });
		
		// [abc[d] -> [a]bcd]
		let mut cursor = Cursor { tail: 0, head: 3 };
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 0, tail: 3 });
		
		// ab[c]d] -> [a]bc]d
		let mut cursor = Cursor { head: 2, tail: 3 };
		cursor.move_previous_word_start();
		assert_eq!(cursor, Cursor { head: 0, tail: 2 });
	}
}
