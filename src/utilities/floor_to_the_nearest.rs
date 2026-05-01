
pub trait Floorable {
	fn floor_to_the_nearest(&mut self, step: Self);
	fn floored_to_the_nearest(self, step: Self) -> Self;
}

impl Floorable for usize {
	fn floor_to_the_nearest(&mut self, step: Self) {
		*self -= *self % step;
	}
	
	fn floored_to_the_nearest(self, step: Self) -> Self {
		self - (self % step)
	}
}

