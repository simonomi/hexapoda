use std::borrow::Cow;
use ratatui::{style::Style, text::Span};

pub const fn empty_span() -> Span<'static> {
	Span {
		style: Style::new(),
		content: Cow::Borrowed("")
	}
}
