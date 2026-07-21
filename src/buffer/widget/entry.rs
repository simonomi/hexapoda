use ratatui::{style::Stylize, text::{Line, Span}};
use crate::buffer::Buffer;

impl Buffer {
	pub fn render_entry<'me>(&'me self, label: &'static str) -> Line<'me> {
		Line::from_iter(
			[" ", label, &self.entry_text, " "]
				.map(Span::from)
				.map(|span| span.white().on_dark_gray())
		)
	}
}
