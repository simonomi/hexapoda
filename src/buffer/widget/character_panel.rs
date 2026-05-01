use std::{borrow::Cow, iter, mem};
use ratatui::{style::{Color, Style, Stylize}, text::Span};
use crate::{buffer::Buffer, utilities::{CustomGreys, empty_span, HasCardinality}, cursor::InCursor};

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
			Some(InCursor::Head) => span.bg(Color::selection_tail_grey()),
			Some(InCursor::Rest) => span.on_dark_gray(),
			None => span,
		}
	}
}
const fn create_character_lookup_table() -> [Span<'static>; u8::CARDINALITY] {
	let mut result = [const { empty_span() }; u8::CARDINALITY];
	
	let mut index = 0;
	while index < u8::CARDINALITY {
		#[allow(clippy::cast_possible_truncation)]
		let byte = index as u8;
		
		result[index].style = style_for_character(byte);
		mem::forget(mem::replace(
			&mut result[index].content,
			content_for_character(byte)
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
