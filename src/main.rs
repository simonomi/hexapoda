#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::cast_possible_truncation)]

use std::{borrow::Cow, cmp::min, env, fs::File, io::Read, iter, mem, process::exit};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use itertools::Itertools;
use ratatui::{style::{Color, Style}, text::{Line, Span, Text}, widgets::Widget};

fn main() {
	let mut app = App::init();
	let mut terminal = ratatui::init();
	
	while !app.should_quit {
		terminal.draw(|frame| {
			frame.render_widget(&app, frame.area());
		}).unwrap();
		
		app.handle_events();
	}
	
	ratatui::restore();
}

#[derive(Debug)]
struct App {
	contents: Vec<u8>,
	scroll_position: usize,
	// cursor_position: usize,
	should_quit: bool
}

impl App {
	fn init() -> Self {
		let input_files: Vec<_> = env::args().skip(1).collect();
		
		if input_files.is_empty() {
			println!("please provide at least one file as input");
			exit(1);
		}
		
		assert!(input_files.len() == 1);
		
		let file_name = input_files.first().unwrap();
		
		let file = File::open(file_name);
		let mut contents = Vec::new();
		file.unwrap().read_to_end(&mut contents).unwrap();
		
		Self {
			contents,
			scroll_position: 0,
			// cursor_position: 0,
			should_quit: false,
		}
	}
	
	fn handle_events(&mut self) {
		match event::read().unwrap() {
			Event::Key(key_event) if key_event.code == KeyCode::Char('q') => {
				self.should_quit = true;
			}
			Event::Key(key_event) if key_event.code == KeyCode::Char('e') &&
			                         key_event.modifiers.contains(KeyModifiers::CONTROL) => {
				let max_scroll_position = self.contents.len() - 0x50;
				self.scroll_position = min(self.scroll_position + BYTES_PER_LINE, max_scroll_position);
			}
			Event::Key(key_event) if key_event.code == KeyCode::Char('y') &&
			                         key_event.modifiers.contains(KeyModifiers::CONTROL) => {
				self.scroll_position = self.scroll_position.saturating_sub(BYTES_PER_LINE);
			}
			_ => {}
		}
	}
}

const BYTES_PER_LINE: usize = 0x10;

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
			.map(|(bytes, address)| render_line(address, bytes))
			.collect();
		
		let text = Text::from(lines);
		
		text.render(area, buf);
	}
}

#[allow(mismatched_lifetime_syntaxes)]
fn render_line(address: usize, bytes: &[u8; BYTES_PER_LINE]) -> Line {
	let spans: Vec<Span<'_>> = iter::once(render_address(address))
		.chain(render_chunks(bytes))
		.chain(iter::once("  ".into()))
		.chain(render_character_panel(bytes))
		.collect();
	
	Line::from(spans)
}

fn render_address(address: usize) -> Span<'static> {
	Span {
		style: Style::new().fg(Color::Rgb(138, 187, 195)),
		content: format!("{address:08x}  ").into()
	}
}

fn render_chunks(bytes: &[u8; BYTES_PER_LINE]) -> impl IntoIterator<Item=Span<'static>> {
	let (chunks, remainder) = bytes.as_chunks::<BYTES_PER_CHUNK>();
	
	assert!(remainder.is_empty());
	
	#[allow(unstable_name_collisions)]
	chunks
		.iter()
		.copied()
		.map(render_chunk)
		.intersperse(vec!["  ".into()])
		.flatten()
}

fn render_character_panel(bytes: &[u8; BYTES_PER_LINE]) -> impl IntoIterator<Item=Span<'static>> {
	bytes
		.iter()
		.copied()
		.map(render_byte_as_character)
}

fn render_byte_as_character(byte: u8) -> Span<'static> {
	const SPAN_FOR_BYTE: [Span; u8::CARDINALITY] = create_byte_character_lookup_table();
	
	SPAN_FOR_BYTE[byte as usize].clone()
}

const fn create_byte_character_lookup_table() -> [Span<'static>; u8::CARDINALITY] {
	let mut result = [const { empty_span() }; u8::CARDINALITY];
	
	let mut index = 0;
	while index < u8::CARDINALITY {
		result[index].style = Style::new().fg(fg_for_byte_as_character(index as u8));
		mem::forget(mem::replace(&mut result[index].content, content_for_character(index as u8)));
		index += 1;
	}
	
	result
}

const fn content_for_character(byte: u8) -> Cow<'static, str> {
	Cow::Borrowed(character_for_byte(byte))
}

const fn character_for_byte(byte: u8) -> &'static str {
	const LOOK_UP_TABLE: [&str; u8::CARDINALITY] = ["⋄", "•", "•", "•", "•", "•", "•", "•", "•", "→", "⏎", "•", "•", "␍", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", "•", " ", "!", "\"", "#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?", "@", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "[", "\\", "]", "^", "_", "`", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "{", "|", "}", "~", "•", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "×", "╳"];
	
	LOOK_UP_TABLE[byte as usize]
}

const fn fg_for_byte_as_character(byte: u8) -> Color {
	match byte {
		b'\0' => Color::Rgb(0xa0, 0xa0, 0xa0),
		b'\t' | b'\n' | b'\r' | b' ' => Color::Rgb(0xfc, 0x6a, 0x5d),
		_ if byte.is_ascii_graphic() => Color::Rgb(0xfc, 0x6a, 0x5d),
		_ if byte.is_ascii() => Color::Rgb(0x50, 0xfa, 0x7b),
		0xFF => Color::White,
		_ => Color::Rgb(0xf1, 0xfa, 0x8c),
	}
}

const BYTES_PER_CHUNK: usize = 4;

fn render_chunk(bytes: [u8; BYTES_PER_CHUNK]) -> Vec<Span<'static>> {
	#[allow(unstable_name_collisions)]
	bytes
		.iter()
		.copied()
		.map(render_byte)
		.intersperse(" ".into())
		.collect()
}

trait HasCardinality {
	const CARDINALITY: usize;
}

impl HasCardinality for u8 {
	const CARDINALITY: usize = 2usize.pow(Self::BITS);
}

fn render_byte(byte: u8) -> Span<'static> {
	const SPAN_FOR_BYTE: [Span; u8::CARDINALITY] = create_lookup_table();
	
	SPAN_FOR_BYTE[byte as usize].clone()
}

const fn create_lookup_table() -> [Span<'static>; u8::CARDINALITY] {
	let mut result = [const { empty_span() }; u8::CARDINALITY];
	
	let mut index = 0;
	while index < u8::CARDINALITY {
		result[index].style = style_for(index as u8);
		mem::forget(mem::replace(&mut result[index].content, content_for(index as u8)));
		index += 1;
	}
	
	result
}

const fn empty_span() -> Span<'static> {
	Span {
		style: Style::new(),
		content: Cow::Borrowed("")
	}
}

const fn style_for(byte: u8) -> Style {
	Style::new().fg(fg_for(byte))
}

const fn fg_for(byte: u8) -> Color {
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

const fn content_for(byte: u8) -> Cow<'static, str> {
	Cow::Borrowed(byte_as_hex(byte))
}

const fn byte_as_hex(byte: u8) -> &'static str {
	const LOOK_UP_TABLE: [&str; u8::CARDINALITY] = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0A", "0B", "0C", "0D", "0E", "0F", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1A", "1B", "1C", "1D", "1E", "1F", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "2A", "2B", "2C", "2D", "2E", "2F", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "3A", "3B", "3C", "3D", "3E", "3F", "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "4A", "4B", "4C", "4D", "4E", "4F", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "5A", "5B", "5C", "5D", "5E", "5F", "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "6A", "6B", "6C", "6D", "6E", "6F", "70", "71", "72", "73", "74", "75", "76", "77", "78", "79", "7A", "7B", "7C", "7D", "7E", "7F", "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "8A", "8B", "8C", "8D", "8E", "8F", "90", "91", "92", "93", "94", "95", "96", "97", "98", "99", "9A", "9B", "9C", "9D", "9E", "9F", "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "AA", "AB", "AC", "AD", "AE", "AF", "B0", "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8", "B9", "BA", "BB", "BC", "BD", "BE", "BF", "C0", "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "CA", "CB", "CC", "CD", "CE", "CF", "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9", "DA", "DB", "DC", "DD", "DE", "DF", "E0", "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9", "EA", "EB", "EC", "ED", "EE", "EF", "F0", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "FA", "FB", "FC", "FD", "FE", "FF"];
	
	LOOK_UP_TABLE[byte as usize]
}
