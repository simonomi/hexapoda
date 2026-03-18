use std::cmp::min;
use crate::{app::App, cursor::Cursor};

#[derive(Debug)]
pub enum EditAction {
	Delete {
		cursor: Cursor,
		old_data: Vec<u8>
	},
	Replace {
		cursor: Cursor,
		old_data: Vec<u8>,
		new_byte: u8
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
			EditAction::Replace {
				cursor, old_data: _, new_byte
			} => self.replace_at_with(*cursor, *new_byte),
		}
	}
	
	fn delete_at(&mut self, cursor: Cursor) {
		self.contents.drain(cursor.range());
		
		self.cursor.head = min(cursor.head, cursor.tail);
		self.cursor.collapse();
	}
	
	fn replace_at_with(&mut self, cursor: Cursor, new_byte: u8) {
		self.contents[self.cursor.range()].fill(new_byte);
		
		self.cursor = cursor;
	}
}
