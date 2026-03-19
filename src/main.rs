#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::cast_possible_truncation)]

use app::App;

mod app;
mod buffer;
mod config;
mod cursor;
mod action;
mod edit_action;

mod cardinality;
mod empty_span;
mod custom_greys;

const BYTES_PER_LINE: usize = 0x10;
const BYTES_PER_CHUNK: usize = 4;
const CHUNKS_PER_LINE: usize = BYTES_PER_LINE / BYTES_PER_CHUNK;

// TODO:
// - multiple buffers (tabs)
// - search
// - modifications
//   - insert/append
//     - mode
//     - how this works with edit history is strange :/
//     - add to edit history when *leaving* insert mode
//   - replace-and-keep-going
//     - mode
//   - change
// - edit character panel
//   - modifier on existing keys like teehee? or jump to panel?
//     - if jump to panel, space?
// - zz/zt/zb
// - visual gg/G
// - jumplist
// - y/p
// - [/] to cycle view offset?
// - J jump to offset
//   - under cursor?

// future directions
// - switch between cursor size u8s/u16s/u32s/u64s?
//   - +/-
// - multi-cursor
//   - s/C
//   - split selection by u8/16/32/etc
// - 'views' for bytes (i8/16/etc u8/16/etc 20.12/8.4/etc)
//   - how to fit??! `-128` longer than `80`
// - mark offsets?
// - utf8?
// - diffing

// when AsciiChar is stabilized, use it instead of char everywhere

fn main() {
	let mut app = App::new();
	let mut terminal = ratatui::init();
	crossterm::terminal::enable_raw_mode().unwrap();
	
	while !app.should_quit {
		terminal.draw(|frame| {
			frame.render_widget(&app, frame.area());
		}).unwrap();
		
		app.handle_events();
	}
	
	crossterm::terminal::disable_raw_mode().unwrap();
	ratatui::restore();
	
	// dbg!(app.edit_history);
	
	for log in app.logs {
		println!("{log}");
	}
}
