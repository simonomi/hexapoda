use std::{cmp::min, iter};
use ratatui::{layout::Rect, text::{Line, Text}, widgets::Widget};
use crate::{BYTES_PER_LINE, buffer::Buffer};

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
		
		for popup in &self.popups {
			if self.scroll_position <= popup.at &&
			   popup.at < self.scroll_position + (hex_area.height.saturating_sub(1) as usize * BYTES_PER_LINE)
			{
				let position_on_screen = popup.at - self.scroll_position;
				let hex_column = position_on_screen % BYTES_PER_LINE;
				
				let popup_area = popup
					.area_at(
						area.x + byte_column_to_screen_column(hex_column) as u16,
						area.y + (position_on_screen / BYTES_PER_LINE) as u16 + 1
					)
					.clamp(hex_area);
				
				popup.clone().render(popup_area, buf);
			}
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

mod address {
	use ratatui::{style::{Color, Style}, text::Span};
	
	pub fn render_address(address: usize) -> Span<'static> {
		Span {
			style: style_for_address(address),
			content: format!("{address:08x}").into()
		}
	}
	
	pub const fn style_for_address(address: usize) -> Style {
		if address.is_multiple_of(0x100) {
			Style::new().fg(Color::Rgb(0x68, 0x99, 0xA0))
		} else {
			Style::new().fg(Color::Rgb(0x8A, 0xBB, 0xC3))
		}
	}
}

mod hex {
	use std::{borrow::Cow, iter::{self, repeat_n}, mem};
	use itertools::Itertools;
use ratatui::{style::{Color, Style, Stylize}, text::Span};
	
	use crate::{BYTES_PER_CHUNK, BYTES_PER_LINE, CHUNKS_PER_LINE, buffer::{Buffer, Mode, PartialAction}, cardinality::HasCardinality, cursor::InCursor, custom_greys::CustomGreys, empty_span::empty_span};
	
	impl Buffer {
		pub fn render_chunks(
			&self,
			address: usize,
			bytes: &[u8; BYTES_PER_LINE]
		) -> impl Iterator<Item=Span<'static>> {
			let (chunks, remainder) = bytes.as_chunks::<BYTES_PER_CHUNK>();
			
			assert!(remainder.is_empty(), "BYTES_PER_LINE should be a multiple of BYTES_PER_CHUNK");
			
			#[allow(unstable_name_collisions)]
			chunks
				.iter()
				.copied()
				.zip((address..).step_by(BYTES_PER_CHUNK))
				.flat_map(|(chunk, address)| {
					self.render_chunk(address, &chunk).collect::<Vec<_>>()
				})
		}
		
		pub fn render_partial_chunks(
			&self,
			address: usize,
			bytes: &[u8]
		) -> impl Iterator<Item=Span<'static>> {
			let (chunks, remainder) = bytes.as_chunks::<BYTES_PER_CHUNK>();
			
			let remainder_address = address + chunks.len() * BYTES_PER_CHUNK;
			#[allow(clippy::if_not_else)]
			let remainder_chunks: Option<Vec<_>> = if !remainder.is_empty() {
				Some(self.render_partial_chunk(remainder_address, remainder).collect())
			} else {
				None
			};
			
			let chunks_rendered = chunks.len() + remainder_chunks.iter().len();
			let chunks_not_rendered = CHUNKS_PER_LINE - chunks_rendered;
			let spaces_per_chunk = BYTES_PER_CHUNK - 1 + 2;
			let bytes_not_rendered = BYTES_PER_LINE - bytes.len();
			
			let padding_width = 2 * bytes_not_rendered +
				spaces_per_chunk * chunks_not_rendered;
			
			#[allow(unstable_name_collisions)]
			chunks
				.iter()
				.copied()
				.zip((address..).step_by(BYTES_PER_CHUNK))
				.map(|(chunk, address)| self.render_chunk(address, &chunk).collect())
				.chain(remainder_chunks)
				.flatten()
				.chain(repeat_n(" ".into(), padding_width))
		}
		
		fn render_chunk(
			&self,
			address: usize,
			bytes: &[u8; BYTES_PER_CHUNK]
		) -> impl Iterator<Item=Span<'static>> {
			iter::once(self.render_large_space_before(address))
				.chain(
					bytes
						.iter()
						.copied()
						.zip(address..)
						.map(|(byte, address)| self.render_byte_at(address, byte))
						.interleave(
							(address..)
								.take(BYTES_PER_CHUNK)
								.skip(1)
								.map(|address| self.render_space_before(address))
						)
				)
		}
		
		fn render_partial_chunk(
			&self,
			address: usize,
			bytes: &[u8]
		) -> impl Iterator<Item=Span<'static>> {
			iter::once(self.render_large_space_before(address))
				.chain(
					bytes
						.iter()
						.copied()
						.zip(address..)
						.map(|(byte, address)| self.render_byte_at(address, byte))
						.interleave(
							(address..)
								.take(BYTES_PER_CHUNK)
								.skip(1)
								.map(|address| self.render_space_before(address))
						)
				)
		}
		
		fn render_byte_at(
			&self,
			address: usize,
			byte: u8
		) -> Span<'static> {
			if self.partial_action == Some(PartialAction::Replace) &&
			   iter::once(&self.primary_cursor)
				.chain(&self.cursors)
				.any(|cursor| cursor.contains(address).is_some())
			{
				let replaced_byte = self.partial_replace.unwrap_or(0) << 4;
				
				self.render_byte_without_replace_preview(address, replaced_byte)
					.black()
			} else {
				self.render_byte_without_replace_preview(address, byte)
			}
		}
		
		fn render_byte_without_replace_preview(
			&self,
			address: usize,
			byte: u8
		) -> Span<'static> {
			const SPAN_FOR_BYTE: [Span; u8::CARDINALITY] = create_byte_lookup_table();
			
			let span = SPAN_FOR_BYTE[byte as usize].clone();
			
			if let Some(place_in_cursor) = self.primary_cursor.contains(address) {
				let head_color = match self.mode {
					Mode::Select => Color::Yellow,
					_ => Color::Gray
				};
				
				match place_in_cursor {
					InCursor::Head => span.bg(head_color),
					InCursor::Rest => span.bg(Color::select_grey()),
				}
			} else {
				match self.cursors
					.iter()
					.find_map(|cursor| cursor.contains(address))
				{
					Some(InCursor::Head) => span.on_gray(),
					Some(InCursor::Rest) => span.bg(Color::select_grey()),
					None => span,
				}
			}
		}
		
		fn render_large_space_before(&self, address: usize) -> Span<'static> {
			let span: Span = if self.marks.contains(&address) {
				" →".into()
			} else {
				"  ".into()
			};
			
			if !address.is_multiple_of(BYTES_PER_LINE) &&
				iter::once(&self.primary_cursor)
					.chain(&self.cursors)
					.any(|cursor| cursor.contains_space_before(address))
			{
				span.bg(Color::select_grey())
			} else {
				span
			}
		}
		
		fn render_space_before(&self, address: usize) -> Span<'static> {
			let span: Span = if self.marks.contains(&address) {
				"→".into()
			} else {
				" ".into()
			};
			
			if iter::once(&self.primary_cursor)
				.chain(&self.cursors)
				.any(|cursor| cursor.contains_space_before(address))
			{
				span.bg(Color::select_grey())
			} else {
				span
			}
		}
	}
	
	const fn create_byte_lookup_table() -> [Span<'static>; u8::CARDINALITY] {
		let mut result = [const { empty_span() }; u8::CARDINALITY];
		
		let mut index = 0;
		while index < u8::CARDINALITY {
			result[index].style = style_for_byte(index as u8);
			mem::forget(mem::replace(&mut result[index].content, content_for_byte(index as u8)));
			index += 1;
		}
		
		result
	}
	
	const fn style_for_byte(byte: u8) -> Style {
		Style::new().fg(fg_for_byte(byte))
	}
	
	const fn fg_for_byte(byte: u8) -> Color {
		match byte {
			0x00       => Color::Rgb(0x80, 0x80, 0x80), // grey
			0x01..0x10 => Color::Rgb(0xFF, 0x71, 0xA9), // red
			0x10..0x20 => Color::Rgb(0xFF, 0x7A, 0x78), // salmon
			0x20..0x30 => Color::Rgb(0xFF, 0x81, 0x23), // red-orange
			0x30..0x40 => Color::Rgb(0xF7, 0x93, 0x00), // yellow-orange
			0x40..0x50 => Color::Rgb(0xE6, 0x9F, 0x00), // yellow
			0x50..0x60 => Color::Rgb(0xC1, 0xB2, 0x00), // green-yellow
			0x60..0x70 => Color::Rgb(0x82, 0xC6, 0x00), // lime
			0x70..0x80 => Color::Rgb(0x00, 0xD5, 0x00), // green
			0x80..0x90 => Color::Rgb(0x00, 0xD4, 0x59), // clover
			0x90..0xA0 => Color::Rgb(0x00, 0xD0, 0x91), // teal
			0xA0..0xB0 => Color::Rgb(0x00, 0xCC, 0xBB), // cyan
			0xB0..0xC0 => Color::Rgb(0x00, 0xC7, 0xDE), // light blue
			0xC0..0xD0 => Color::Rgb(0x00, 0xBE, 0xFF), // blue
			0xD0..0xE0 => Color::Rgb(0x6C, 0xAF, 0xFF), // blurple
			0xE0..0xF0 => Color::Rgb(0xB2, 0x98, 0xFF), // purple
			0xF0..0xFF => Color::Rgb(0xFF, 0x4D, 0xFF), // pink
			0xFF       => Color::White
		}
	}
	
	const fn content_for_byte(byte: u8) -> Cow<'static, str> {
		Cow::Borrowed(hex_for_byte(byte))
	}
	
	const fn hex_for_byte(byte: u8) -> &'static str {
		const LOOK_UP_TABLE: [&str; u8::CARDINALITY] = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0a", "0b", "0c", "0d", "0e", "0f", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b", "1c", "1d", "1e", "1f", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "2a", "2b", "2c", "2d", "2e", "2f", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "3a", "3b", "3c", "3d", "3e", "3f", "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "4a", "4b", "4c", "4d", "4e", "4f", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "5a", "5b", "5c", "5d", "5e", "5f", "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "6a", "6b", "6c", "6d", "6e", "6f", "70", "71", "72", "73", "74", "75", "76", "77", "78", "79", "7a", "7b", "7c", "7d", "7e", "7f", "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "8a", "8b", "8c", "8d", "8e", "8f", "90", "91", "92", "93", "94", "95", "96", "97", "98", "99", "9a", "9b", "9c", "9d", "9e", "9f", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "aa", "ab", "ac", "ad", "ae", "af", "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "b9", "ba", "bb", "bc", "bd", "be", "bf", "c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8", "c9", "ca", "cb", "cc", "cd", "ce", "cf", "d0", "d1", "d2", "d3", "d4", "d5", "d6", "d7", "d8", "d9", "da", "db", "dc", "dd", "de", "df", "e0", "e1", "e2", "e3", "e4", "e5", "e6", "e7", "e8", "e9", "ea", "eb", "ec", "ed", "ee", "ef", "f0", "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "fa", "fb", "fc", "fd", "fe", "ff"];
		
		LOOK_UP_TABLE[byte as usize]
	}
}

mod character_panel {
	use std::{borrow::Cow, iter, mem};
	use ratatui::{style::{Color, Style, Stylize}, text::Span};
	use crate::{buffer::Buffer, cardinality::HasCardinality, cursor::InCursor, custom_greys::CustomGreys, empty_span::empty_span};
	
	impl Buffer {
		pub fn render_character_panel(
			&self,
			address: usize,
			bytes: &[u8]
		) -> impl Iterator<Item=Span<'static>> {
			bytes
				.iter()
				.copied()
				.zip(address..)
				.map(|(byte, address)| self.render_character_at(address, byte))
		}
		
		fn render_character_at(
			&self,
			address: usize,
			byte: u8
		) -> Span<'static> {
			const SPAN_FOR_BYTE: [Span; u8::CARDINALITY] = create_character_lookup_table();
			
			let span = SPAN_FOR_BYTE[byte as usize].clone();
			
			match iter::once(&self.primary_cursor)
				.chain(&self.cursors)
				.find_map(|cursor| cursor.contains(address))
			{
				Some(InCursor::Head) => span.bg(Color::select_grey()),
				Some(InCursor::Rest) => span.on_dark_gray(),
				None => span,
			}
		}
	}
	const fn create_character_lookup_table() -> [Span<'static>; u8::CARDINALITY] {
		let mut result = [const { empty_span() }; u8::CARDINALITY];
		
		let mut index = 0;
		while index < u8::CARDINALITY {
			result[index].style = style_for_character(index as u8);
			mem::forget(mem::replace(
				&mut result[index].content,
				content_for_character(index as u8)
			));
			index += 1;
		}
		
		result
	}
	
	const fn style_for_character(byte: u8) -> Style {
		Style::new().fg(fg_for_character(byte))
	}
	
	const fn fg_for_character(byte: u8) -> Color {
		match byte {
			b'\0' => Color::Rgb(0xA0, 0xA0, 0xA0), // grey
			b'\t' | b'\n' | b'\r' | b' ' => Color::Red,
			_ if byte.is_ascii_graphic() => Color::Red,
			_ if byte.is_ascii() => Color::Green,
			0xFF => Color::White,
			_ => Color::Yellow,
		}
	}
	
	const fn content_for_character(byte: u8) -> Cow<'static, str> {
		Cow::Borrowed(character_for_byte(byte))
	}
	
	const fn character_for_byte(byte: u8) -> &'static str {
		const LOOK_UP_TABLE: [&str; u8::CARDINALITY] = ["⋄", "•", "•", "•", "•", "•", "•", "•", "•", "→", "⏎", "•", "•", "␍", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", " ", "!", "\"", "#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?", "@", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "[", "\\", "]", "^", "_", "`", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "{", "|", "}", "~", "•", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "╳"];
		
		LOOK_UP_TABLE[byte as usize]
	}
}

mod status_line {
	use crate::{buffer::Buffer, custom_greys::CustomGreys};
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
}

mod extra_statuses {
	use crate::buffer::Buffer;
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
}

fn byte_column_to_screen_column(byte_column: usize) -> usize {
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
