use std::{cmp::min, iter};
use ratatui::{text::{Line, Span, Text}, widgets::Widget};

use crate::{BYTES_PER_LINE, app::App};

impl Widget for &App {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
		let screen_end = self.scroll_position + BYTES_PER_LINE * (area.height as usize);
		let bytes_end = min(screen_end, self.contents.len());
		
		// TODO: bounds check this
		let bytes_to_render = &self.contents[self.scroll_position..bytes_end];
		
		let (chunks, remainder) = bytes_to_render
			.as_chunks::<BYTES_PER_LINE>();
		
		assert!(remainder.is_empty());
		
		let lines: Vec<_> = chunks
			.iter()
			.zip((self.scroll_position..).step_by(BYTES_PER_LINE))
			.map(|(bytes, address)| self.render_line(address, bytes))
			.collect();
		
		let text = Text::from(lines);
		
		text.render(area, buf);
	}
}

impl App {
	#[allow(mismatched_lifetime_syntaxes)]
	fn render_line(&self, address: usize, bytes: &[u8; BYTES_PER_LINE]) -> Line {
		let spans: Vec<Span<'_>> = iter::once(address::render_address(address))
			.chain(self.render_chunks(address, bytes))
			.chain(iter::once("  ".into()))
			.chain(character_panel::render_character_panel(bytes))
			.collect();
	
		Line::from(spans)
	}
}

mod address {
	use ratatui::{style::{Color, Style}, text::Span};
	
	pub fn render_address(address: usize) -> Span<'static> {
		Span {
			style: Style::new().fg(Color::Rgb(138, 187, 195)),
			content: format!("{address:08x}  ").into()
		}
	}
}

mod hex {
	use std::{borrow::Cow, mem};
	use itertools::Itertools;
	use ratatui::{style::{Color, Style, Stylize}, text::Span};
	
	use crate::{BYTES_PER_CHUNK, BYTES_PER_LINE, CHUNKS_PER_LINE, app::App, cardinality::HasCardinality, cursor::InCursor, empty_span::empty_span, select_grey::SelectGrey};
	
	impl App {
		pub fn render_chunks(
			&self,
			address: usize,
			bytes: &[u8; BYTES_PER_LINE]
		) -> impl IntoIterator<Item=Span<'static>> {
			let (chunks, remainder) = bytes.as_chunks::<BYTES_PER_CHUNK>();
			
			assert!(remainder.is_empty());
			
			#[allow(unstable_name_collisions)]
			chunks
				.iter()
				.copied()
				.zip((address..).step_by(BYTES_PER_CHUNK))
				.map(|(chunk, address)| self.render_chunk(address, chunk))
				.interleave(
					(address..)
						.step_by(BYTES_PER_CHUNK)
						.take(CHUNKS_PER_LINE)
						.skip(1)
						.map(|address| vec![self.render_large_space_before(address)])
				)
				.flatten()
		}
		
		fn render_chunk(
			&self,
			address: usize,
			bytes: [u8; BYTES_PER_CHUNK]
		) -> Vec<Span<'static>> {
			#[allow(unstable_name_collisions)]
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
				.collect()
		}
		
		fn render_byte_at(
			&self,
			address: usize,
			byte: u8
		) -> Span<'static> {
			const SPAN_FOR_BYTE: [Span; u8::CARDINALITY] = create_byte_lookup_table();
			
			let span = SPAN_FOR_BYTE[byte as usize].clone();
			
			match self.cursor.contains(address) {
				Some(InCursor::Head) => span.bg(Color::Gray),
				Some(InCursor::Rest) => span.bg(Color::select_grey()),
				None => span,
			}
		}
		
		fn render_large_space_before(&self, address: usize) -> Span<'static> {
			if self.cursor.contains_space_before(address) {
				Span {
					style: Style::new().bg(Color::select_grey()),
					content: "  ".into()
				}
			} else {
				"  ".into()
			}
		}
		
		fn render_space_before(&self, address: usize) -> Span<'static> {
			if self.cursor.contains_space_before(address) {
				Span {
					style: Style::new().bg(Color::select_grey()),
					content: " ".into()
				}
			} else {
				" ".into()
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
			0x00       => Color::Rgb(0xA0, 0xA0, 0xA0), // grey
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
		const LOOK_UP_TABLE: [&str; u8::CARDINALITY] = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0A", "0B", "0C", "0D", "0E", "0F", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1A", "1B", "1C", "1D", "1E", "1F", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "2A", "2B", "2C", "2D", "2E", "2F", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "3A", "3B", "3C", "3D", "3E", "3F", "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "4A", "4B", "4C", "4D", "4E", "4F", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "5A", "5B", "5C", "5D", "5E", "5F", "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "6A", "6B", "6C", "6D", "6E", "6F", "70", "71", "72", "73", "74", "75", "76", "77", "78", "79", "7A", "7B", "7C", "7D", "7E", "7F", "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "8A", "8B", "8C", "8D", "8E", "8F", "90", "91", "92", "93", "94", "95", "96", "97", "98", "99", "9A", "9B", "9C", "9D", "9E", "9F", "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "AA", "AB", "AC", "AD", "AE", "AF", "B0", "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8", "B9", "BA", "BB", "BC", "BD", "BE", "BF", "C0", "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "CA", "CB", "CC", "CD", "CE", "CF", "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9", "DA", "DB", "DC", "DD", "DE", "DF", "E0", "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9", "EA", "EB", "EC", "ED", "EE", "EF", "F0", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "FA", "FB", "FC", "FD", "FE", "FF"];
		
		LOOK_UP_TABLE[byte as usize]
	}
}

mod character_panel {
	use std::{borrow::Cow, mem};
	use ratatui::{style::{Color, Style}, text::Span};
	
	use crate::{BYTES_PER_LINE, cardinality::HasCardinality, empty_span::empty_span};
	
	pub fn render_character_panel(
		bytes: &[u8; BYTES_PER_LINE]
	) -> impl IntoIterator<Item=Span<'static>> {
		bytes
			.iter()
			.copied()
			.map(render_character)
	}
	
	fn render_character(byte: u8) -> Span<'static> {
		const SPAN_FOR_BYTE: [Span; u8::CARDINALITY] = create_character_lookup_table();
		
		SPAN_FOR_BYTE[byte as usize].clone()
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
