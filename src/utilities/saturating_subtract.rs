pub trait SaturatingSubtract {
	fn saturating_subtract(&mut self, other: Self);
}

impl SaturatingSubtract for usize {
	fn saturating_subtract(&mut self, other: Self) {
		*self = self.saturating_sub(other);
	}
}
