use std::borrow::Cow;
use ratatui::{style::Style, text::Span};

// this can't just use Span::default() because it needs to be const
pub const fn empty_span() -> Span<'static> {
	Span {
		style: Style::new(),
		content: Cow::Borrowed("")
	}
}
