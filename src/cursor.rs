use std::{cmp::{max, min}, mem::swap, ops::RangeInclusive};

mod actions;

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
	
	pub fn clamp(&mut self, min: usize, max: usize) {
		self.head = self.head.clamp(min, max);
		self.tail = self.tail.clamp(min, max);
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
}
