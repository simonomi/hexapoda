use std::{cmp::{max, min}, mem::swap, ops::RangeInclusive};

use crate::BYTES_PER_LINE;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
	pub head: usize,
	pub tail: usize
}

pub enum InCursor {
	Head,
	Rest
}

impl Cursor {
	pub const fn at(index: usize) -> Self {
		Self { head: index, tail: index }
	}
	
	pub fn lower_bound(&self) -> usize {
		min(self.head, self.tail)
	}
	
	pub fn upper_bound(&self) -> usize {
		max(self.head, self.tail)
	}
	
	pub fn range(&self) -> RangeInclusive<usize> {
		self.lower_bound()..=self.upper_bound()
	}
	
	pub fn len(&self) -> usize {
		self.upper_bound() - self.lower_bound() + 1
	}
	
	pub const fn contains(&self, index: usize) -> Option<InCursor> {
		if index == self.head {
			Some(InCursor::Head)
		} else if (self.head < index && index <= self.tail) ||
		          (self.tail <= index && index < self.head)
		{
			Some(InCursor::Rest)
		} else {
			None
		}
	}
	
	pub const fn contains_space_before(&self, index: usize) -> bool {
		(self.head < index && index <= self.tail) ||
		(self.tail < index && index <= self.head)
	}
	
	pub const fn collapse(&mut self) {
		self.tail = self.head;
	}
	
	pub const fn flip(&mut self) {
		swap(&mut self.head, &mut self.tail);
	}
	
	// TODO: in visual mode, should only clamp head
	pub fn clamp(&mut self, scroll_position: usize, screen_size: usize) {
		let max_row = scroll_position + screen_size - 1;
		
		self.head = self.head.clamp(scroll_position, max_row);
		self.tail = self.tail.clamp(scroll_position, max_row);
	}
	
	pub fn combine_with(&mut self, other: Self) {
		if self.head < self.tail {
			self.head = min(self.head, other.head);
			self.tail = max(self.tail, other.tail);
		} else {
			self.head = max(self.head, other.head);
			self.tail = min(self.tail, other.tail);
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
		self.head -= self.head % BYTES_PER_LINE;
		self.collapse();
	}
	
	pub fn goto_line_end(&mut self, max: usize) {
		self.head = min(
			self.head + BYTES_PER_LINE - 1 - (self.head % BYTES_PER_LINE),
			max
		);
		self.collapse();
	}
	
	pub const fn goto_file_start(&mut self) {
		self.head %= BYTES_PER_LINE;
		self.collapse();
	}
	
	pub const fn goto_file_end(&mut self, max: usize) {
		self.head = previous_multiple_of(BYTES_PER_LINE, max + 1) +
			(self.head % BYTES_PER_LINE);
		
		self.collapse();
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
			self.head += BYTES_PER_LINE - 1 - (self.head % BYTES_PER_LINE);
		}
	}
	
	pub const fn extend_line_above(&mut self, max: usize) {
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
			self.tail += BYTES_PER_LINE - 1 - (self.tail % BYTES_PER_LINE);
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
		cursor.move_to_next_word(99);
		assert_eq!(cursor, Cursor::at(4));
		
		// a[b]cd efgh -> abcd [e]fgh
		let mut cursor = Cursor::at(1);
		cursor.move_to_next_word(99);
		assert_eq!(cursor, Cursor::at(4));
		
		// ab[c]d efgh -> abcd [e]fgh
		let mut cursor = Cursor::at(2);
		cursor.move_to_next_word(99);
		assert_eq!(cursor, Cursor::at(4));
		
		// abc[d] efgh -> abcd [e]fgh
		let mut cursor = Cursor::at(3);
		cursor.move_to_next_word(99);
		assert_eq!(cursor, Cursor::at(4));
		
		// [a]bcd -> abc[d]
		let mut cursor = Cursor::at(0);
		cursor.move_to_next_word(3);
		assert_eq!(cursor, Cursor::at(3));
		
		// [a]bc -> ab[c]
		let mut cursor = Cursor::at(0);
		cursor.move_to_next_word(2);
		assert_eq!(cursor, Cursor::at(2));
		
		// [a]b -> a[b]
		let mut cursor = Cursor::at(0);
		cursor.move_to_next_word(1);
		assert_eq!(cursor, Cursor::at(1));
		
		// [a] -> [a]
		let mut cursor = Cursor::at(0);
		cursor.move_to_next_word(0);
		assert_eq!(cursor, Cursor::at(0));
		
		// ab[c]d -> abc[d]
		let mut cursor = Cursor::at(2);
		cursor.move_to_next_word(3);
		assert_eq!(cursor, Cursor::at(3));
		
		// abc[d] -> abc[d]
		let mut cursor = Cursor::at(3);
		cursor.move_to_next_word(3);
		assert_eq!(cursor, Cursor::at(3));
		
		// ab[c[d] -> ab[c[d]
		let mut cursor = Cursor { tail: 2, head: 3 };
		cursor.move_to_next_word(3);
		assert_eq!(cursor, Cursor { tail: 2, head: 3 });
	}
	
	#[test]
	fn next_end() {
		// [a]bcd -> [abcd]
		let mut cursor = Cursor::at(0);
		cursor.move_to_next_end(99);
		assert_eq!(cursor, Cursor { tail: 0, head: 3 });
		
		// a[b]cd -> [abcd]
		let mut cursor = Cursor::at(1);
		cursor.move_to_next_end(99);
		assert_eq!(cursor, Cursor { tail: 1, head: 3 });
		
		// ab[c]d -> [abcd]
		let mut cursor = Cursor::at(2);
		cursor.move_to_next_end(99);
		assert_eq!(cursor, Cursor { tail: 2, head: 3 });
		
		// abc[d] efgh -> abcd [efgh]
		let mut cursor = Cursor::at(3);
		cursor.move_to_next_end(99);
		assert_eq!(cursor, Cursor { tail: 4, head: 7 });
		
		// abcd [e]fgh -> abcd [efgh]
		let mut cursor = Cursor::at(4);
		cursor.move_to_next_end(99);
		assert_eq!(cursor, Cursor { tail: 4, head: 7 });
		
		// abcd e[f]gh -> abcd e[fgh]
		let mut cursor = Cursor::at(5);
		cursor.move_to_next_end(99);
		assert_eq!(cursor, Cursor { tail: 5, head: 7 });
		
		// abcd ef[g]h -> abcd ef[gh]
		let mut cursor = Cursor::at(6);
		cursor.move_to_next_end(99);
		assert_eq!(cursor, Cursor { tail: 6, head: 7 });
		
		// abcd efg[h] ijkl -> abcd efgh [ijkl]
		let mut cursor = Cursor::at(7);
		cursor.move_to_next_end(99);
		assert_eq!(cursor, Cursor { tail: 8, head: 11 });
		
		// abcd efg[h] -> abcd efg[h]
		let mut cursor = Cursor::at(7);
		cursor.move_to_next_end(7);
		assert_eq!(cursor, Cursor { tail: 7, head: 7 });
		
		// abcd e[fgh] -> abcd e[fgh]
		let mut cursor = Cursor { tail: 5, head: 7 };
		cursor.move_to_next_end(7);
		assert_eq!(cursor, Cursor { tail: 5, head: 7 });
		
		// a[b]c -> a[bc]
		let mut cursor = Cursor::at(1);
		cursor.move_to_next_end(2);
		assert_eq!(cursor, Cursor { tail: 1, head: 2 });
		
		// a[bc] -> a[bc]
		let mut cursor = Cursor { tail: 1, head: 2};
		cursor.move_to_next_end(2);
		assert_eq!(cursor, Cursor { tail: 1, head: 2 });
		
		// a[b] -> a[b]
		let mut cursor = Cursor::at(1);
		cursor.move_to_next_end(1);
		assert_eq!(cursor, Cursor::at(1));
		
		// [a]b -> [ab]
		let mut cursor = Cursor::at(0);
		cursor.move_to_next_end(1);
		assert_eq!(cursor, Cursor { tail: 0, head: 1 });
		
		// [ab] -> [ab]
		let mut cursor = Cursor { tail: 0, head: 1};
		cursor.move_to_next_end(1);
		assert_eq!(cursor, Cursor { tail: 0, head: 1 });
		
		// [a] -> [a]
		let mut cursor = Cursor::at(0);
		cursor.move_to_next_end(0);
		assert_eq!(cursor, Cursor::at(0));
		
		// [a]bcd] -> [abc[d]
		let mut cursor = Cursor { head: 0, tail: 3 };
		cursor.move_to_next_end(99);
		assert_eq!(cursor, Cursor { tail: 0, head: 3 });
		
		// [a[b]cd -> a[bc[d]
		let mut cursor = Cursor { tail: 0, head: 1 };
		cursor.move_to_next_end(99);
		assert_eq!(cursor, Cursor { tail: 1, head: 3 });
		
		// abc[d] ef -> abcd [ef]
		let mut cursor = Cursor::at(3);
		cursor.move_to_next_end(5);
		assert_eq!(cursor, Cursor { tail: 4, head: 5 });
	}
	
	#[test]
	fn previous_beginning() {
		// abcd efgh [i]jkl -> abcd [efgh] ijkl
		let mut cursor = Cursor::at(8);
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 4, tail: 7 });
		
		// abcd efg[h] -> abcd [efgh]
		let mut cursor = Cursor::at(7);
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 4, tail: 7 });
		
		// abcd ef[g]h -> abcd [efg]h
		let mut cursor = Cursor::at(6);
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 4, tail: 6 });
		
		// abcd e[f]gh -> abcd [ef]gh
		let mut cursor = Cursor::at(5);
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 4, tail: 5 });
		
		// abcd [e]fgh -> [abcd] efgh
		let mut cursor = Cursor::at(4);
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 0, tail: 3 });
		
		// abc[d] -> [abcd]
		let mut cursor = Cursor::at(3);
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 0, tail: 3 });
		
		// ab[c]d -> [abc]d
		let mut cursor = Cursor::at(2);
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 0, tail: 2 });
		
		// a[b]cd -> [ab]cd
		let mut cursor = Cursor::at(1);
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 0, tail: 1 });
		
		// [a]bcd -> [a]bcd
		let mut cursor = Cursor::at(0);
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 0, tail: 0 });
		
		// [abc[d] -> [a]bcd]
		let mut cursor = Cursor { tail: 0, head: 3 };
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 0, tail: 3 });
		
		// ab[c]d] -> [a]bc]d
		let mut cursor = Cursor { head: 2, tail: 3 };
		cursor.move_to_previous_beginning();
		assert_eq!(cursor, Cursor { head: 0, tail: 2 });
	}
}
