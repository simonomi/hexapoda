use std::cmp::min;
use crate::{app::App, cursor::Cursor};

pub enum EditAction {
	Delete {
		cursor: Cursor,
		data: Vec<u8>
	}
}

impl App {
	pub fn execute_and_add(&mut self, edit_action: EditAction) {
		self.execute_edit(&edit_action);
		self.edit_history.push(edit_action);
	}
	
	fn execute_edit(&mut self, edit_action: &EditAction) {
		match edit_action {
			EditAction::Delete { cursor, .. } => self.delete_at(*cursor),
		}
	}
	
	fn delete_at(&mut self, cursor: Cursor) {
		self.contents.drain(cursor.range());
		
		self.cursor.head = min(cursor.head, cursor.tail);
		self.cursor.collapse();
	}
}
