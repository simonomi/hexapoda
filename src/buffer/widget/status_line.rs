use crate::{buffer::{Buffer, Mode}, custom_greys::CustomGreys};
use ratatui::{style::{Color, Stylize}, text::{Line, Span, Text}};

impl Buffer {
	pub fn render_status_line(&self) -> Text<'_> {
		Text::from(
			Line::from_iter([
				self.render_mode(),
				" ".into(),
				self.render_file_name(),
				self.modified_indicator(),
				"  ".into(),
				self.alert_message.clone()
			])
		)
		.bg(Color::ui_grey())
	}
	
	fn render_mode(&self) -> Span<'static> {
		Span::from(self.mode.label())
			.black()
			.bg(self.mode.color())
	}
	
	fn render_file_name(&self) -> Span<'_> {
		Span::from(&self.file_name)
	}
	
	fn modified_indicator(&self) -> Span<'static> {
		if self.has_unsaved_changes() {
			" [+]".into()
		} else {
			"".into()
		}
	}
}

impl Mode {
	pub const fn label(self) -> &'static str {
		match self {
			Self::Normal => " NORMAL ",
			Self::Select => " SELECT ",
			// Self::Insert => " INSERT ",
		}
	}
	
	pub const fn color(self) -> Color {
		match self {
			Self::Normal => Color::Blue,
			Self::Select => Color::Yellow,
			// Self::Insert => Color::Green,
		}
	}
}
