use ratatui::{layout::Rect, style::{Color, Stylize}, text::{Line, Span}, widgets::Widget};
use crate::{app::App, buffer::Buffer, custom_greys::CustomGreys};

impl Widget for &App {
	fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
		if self.buffers.len() == 1 {
			self.current_buffer().render(area, buf);
		} else {
			let tab_bar_area =  Rect::new(area.x, area.y, area.width, 1);
			self.render_tab_bar().render(tab_bar_area, buf);
			
			let buffer_area = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
			self.current_buffer().render(buffer_area, buf);
		}
	}
}

impl App {
	fn render_tab_bar(&self) -> Line<'static> {
		self.buffers
			.iter()
			.enumerate()
			.map(|(index, buffer)| self.tab_for(buffer, index == self.current_buffer_index))
			.collect()
	}
	
	fn tab_for(&self, buffer: &Buffer, is_active: bool) -> Span<'static> {
		let background = if is_active {
			Color::select_grey()
		} else {
			Color::ui_grey()
		};
		
		Span::from(format!(" {} ", buffer.file_name))
			.bg(background)
	}
}
