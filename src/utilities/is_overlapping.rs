use std::ops::RangeInclusive;

pub trait IsOverlapping {
	fn is_overlapping(&self, other: &Self) -> bool;
}

impl IsOverlapping for RangeInclusive<usize> {
	fn is_overlapping(&self, other: &Self) -> bool {
		self.contains(other.start()) || self.contains(other.end())
	}
}
