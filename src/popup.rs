use ratatui::{layout::{Constraint, Rect}, style::{Style, Stylize}, text::Span, widgets::{Block, Borders, Clear, Widget}};

#[derive(Clone)]
pub struct Popup {
	pub at: usize,
	width: u16,
	primary: bool,
	lines: Vec<Span<'static>>
}

impl Popup {
	pub fn new(at: usize, lines: Vec<Span<'static>>) -> Self {
		Self {
			at,
			width: lines
				.iter()
				.map(|line| u16::try_from(line.width()).unwrap())
				.max()
				.unwrap_or(0),
			primary: false,
			lines
		}
	}
	
	pub fn area_at(&self, x: u16, y: u16) -> Rect {
		Rect {
			x,
			y,
			width: self.width + 2,
			height: u16::try_from(self.lines.len()).unwrap()
		}
	}
	
	#[allow(clippy::wrong_self_convention)]
	pub const fn as_primary(mut self) -> Self {
		self.primary = true;
		self
	}
}

impl Widget for Popup {
	fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
		Clear.render(area, buf);
		
		let border_color = if self.primary {
			Style::new().white()
		} else {
			Style::new().gray()
		};
		
		Block::new()
			.on_dark_gray()
			.borders(Borders::LEFT | Borders::RIGHT)
			.border_style(border_color)
			.render(area, buf);
		
		for (line, area) in self.lines.iter().zip(area.rows()) {
			line.render(
				area.centered_horizontally(
					Constraint::Length(u16::try_from(line.width()).unwrap())
				),
				buf
			);
		}
	}
}
