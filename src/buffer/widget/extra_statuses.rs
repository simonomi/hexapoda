use crate::buffer::{Buffer, PartialAction};
use ratatui::text::Line;

impl Buffer {
	pub fn render_extra_statuses(&self) -> Line<'_> {
		let partial_action = self.partial_action
			.as_ref()
			.map_or("", |partial_action| partial_action.label());
		
		if self.contents.is_empty() {
			format!("{partial_action} ").into()
		} else {
			#[allow(clippy::cast_precision_loss)]
			let percentage = self.primary_cursor.head as f64 / self.max_contents_index() as f64 * 100.0;
			
			format!("{partial_action} {percentage:.0}% ").into()
		}
	}
}

impl PartialAction {
	pub const fn label(self) -> &'static str {
		use PartialAction::*;
		
		match self {
			Goto => "g",
			View => "z",
			Replace => "r",
			Space => "␠",
			Repeat => "×",
			Till => "t",
		}
	}
}
