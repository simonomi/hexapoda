#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::cast_possible_truncation)]

use app::App;

mod cardinality;
mod empty_span;
mod custom_greys;
mod app;
mod cursor;

const BYTES_PER_LINE: usize = 0x10;
const BYTES_PER_CHUNK: usize = 4;
const CHUNKS_PER_LINE: usize = BYTES_PER_LINE / BYTES_PER_CHUNK;

// TODO:
// - modes
//   - select
//   - insert
//   - zz/zt/zb
// - search
// - jumplist
// - modifications
//   - insert/append
//     - mode
//   - replace
//     - partial action
//   - replace-and-keep-going
//     - mode
//   - delete
//   - change

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
	
	for log in app.logs {
		println!("{log}");
	}
}
