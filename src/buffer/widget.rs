use std::{cmp::min, iter};
use ratatui::{layout::Rect, text::{Line, Text}, widgets::Widget};
use crate::{BYTES_PER_LINE, buffer::Buffer};

mod address;
mod hex;
mod character_panel;
mod status_line;
mod extra_statuses;

impl Widget for &Buffer {
	fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
		let screen_end = self.scroll_position + BYTES_PER_LINE * (area.height as usize - 1);
		let bytes_end = min(screen_end, self.contents.len());
		
		let bytes_to_render = &self.contents[self.scroll_position..bytes_end];
		
		let (chunks, remainder) = bytes_to_render
			.as_chunks::<BYTES_PER_LINE>();
		
		let hex_lines = chunks
			.iter()
			.zip((self.scroll_position..).step_by(BYTES_PER_LINE))
			.map(|(bytes, address)| self.render_line(address, bytes));
		
		let remainder_address = bytes_end - remainder.len();
		#[allow(clippy::if_not_else)]
		let remainder_line = if !remainder.is_empty() {
			Some(self.render_partial_line(remainder_address, remainder))
		} else {
			None
		};
		
		let hex_text: Text = hex_lines
			.chain(remainder_line)
			.collect();
		
		let hex_area = Rect::new(area.x, area.y, area.width, area.height - 1);
		hex_text.render(hex_area, buf);
		
		if self.contents.is_empty() {
			Line::from("empty file").render(area, buf);
		}
		
		let status_line_area = Rect::new(area.x, area.bottom() - 1, area.width, 1);
		self.render_status_line().render(status_line_area, buf);
		
		self.render_extra_statuses()
			.right_aligned()
			.render(status_line_area, buf);
		
		let mut primary_popup = None;
		let mut primary_popup_area = None;
		
		for popup in &self.popups {
			if self.scroll_position <= popup.at &&
			   popup.at < self.scroll_position + (hex_area.height.saturating_sub(1) as usize * BYTES_PER_LINE)
			{
				let position_on_screen = popup.at - self.scroll_position;
				let hex_column = position_on_screen % BYTES_PER_LINE;
				
				let popup_area = popup
					.area_at(
						area.x + byte_column_to_screen_column(hex_column),
						area.y + u16::try_from(position_on_screen / BYTES_PER_LINE).unwrap() + 1
					)
					.clamp(hex_area);
				
				if popup.at == self.primary_cursor.lower_bound() {
					primary_popup = Some(popup);
					primary_popup_area = Some(popup_area);
				}
				
				popup.clone().render(popup_area, buf);
			}
		}
		
		if let Some(primary_popup) = primary_popup &&
		   let Some(primary_popup_area) = primary_popup_area
		{
			primary_popup.clone().as_primary().render(primary_popup_area, buf);
		}
		
		// if self.partial_action == Some(PartialAction::Space) {
		// 	let input_field_area = Rect::new(area.x, area.bottom() - 2, area.width, 1);
		// 	Span::from("/0F673 ")
		// 		.on_dark_gray()
		// 		.render(input_field_area, buf);
		// }
	}
}

impl Buffer {
	fn render_line(&self, address: usize, bytes: &[u8; BYTES_PER_LINE]) -> Line<'static> {
		iter::once(address::render_address(address))
			.chain(self.render_chunks(address, bytes))
			.chain(iter::once("  ".into()))
			.chain(self.render_character_panel(address, bytes))
			.collect()
	}
	
	fn render_partial_line(&self, address: usize, bytes: &[u8]) -> Line<'static> {
		iter::once(address::render_address(address))
			.chain(self.render_partial_chunks(address, bytes))
			.chain(iter::once("  ".into()))
			.chain(self.render_character_panel(address, bytes))
			.collect()
	}
}

fn byte_column_to_screen_column(byte_column: usize) -> u16 {
	match byte_column {
		0 => 10,
		1 => 13,
		2 => 16,
		3 => 19,
		
		4 => 23,
		5 => 26,
		6 => 29,
		7 => 32,
		
		8 => 36,
		9 => 39,
		10 => 42,
		11 => 45,
		
		12 => 49,
		13 => 52,
		14 => 55,
		15 => 58,
		
		_ => panic!("byte column must be less than 16"),
	}
}
