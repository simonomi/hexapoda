use std::cmp::min;
use crate::{app::App, cursor::Cursor};

#[derive(Debug)]
pub enum EditAction {
	// this is a hacky workaround to allow disjoint borrows for
	// undoing/redoing edits. because the undo/redo operations need
	// to borrow an EditAction from edit_history, but should never touch
	// edit_history themselves, we can swap out the action in question,
	// then swap it back in once the undo/redo is done
	Placeholder,
	Delete {
		cursor: Cursor,
		old_data: Vec<u8>
	},
	Replace {
		cursor: Cursor,
		old_data: Vec<u8>,
		new_byte: u8
	},
	// Insert {
	// 	cursor: Cursor,
	//  which side of cursor? append/insert
	// 	new_data: Vec<u8>
	// }
}

impl App {
	pub fn execute_and_add(&mut self, edit_action: EditAction) {
		assert!(!matches!(edit_action, EditAction::Placeholder));
		
		self.execute_edit(&edit_action);
		
		if let Some(date) = self.time_traveling {
			self.edit_history.truncate(date);
			self.time_traveling = None;
			
			if self.last_saved_at.is_some_and(|it| it > date) {
				self.last_saved_at = None;
			}
		}
		
		self.edit_history.push(edit_action);
	}
	
	pub fn execute_edit(&mut self, edit_action: &EditAction) {
		match edit_action {
			EditAction::Placeholder => unreachable!(),
			EditAction::Delete { cursor, .. } => self.delete_at(*cursor),
			EditAction::Replace {
				cursor, old_data: _, new_byte
			} => self.replace_at_with(*cursor, *new_byte),
		}
	}
	
	pub fn undo_edit(&mut self, edit_action: &EditAction) {
		match edit_action {
			EditAction::Placeholder => unreachable!(),
			EditAction::Delete { cursor, old_data } => self.undo_delete_at(*cursor, old_data),
			EditAction::Replace {
				cursor, old_data, ..
			} => self.undo_replace_at_with(*cursor, old_data),
		}
	}
	
	fn delete_at(&mut self, cursor: Cursor) {
		self.contents.drain(cursor.range());
		
		self.cursor.head = min(min(cursor.head, cursor.tail), self.contents.len() - 1);
		self.cursor.collapse();
	}
	
	fn undo_delete_at(&mut self, cursor: Cursor, old_data: &[u8]) {
		let cursor_start = min(cursor.head, cursor.tail);
		
		self.contents.splice(
			cursor_start..cursor_start,
			old_data.iter().copied()
		);
		
		self.cursor = cursor;
	}
	
	fn replace_at_with(&mut self, cursor: Cursor, new_byte: u8) {
		self.contents[cursor.range()].fill(new_byte);
		
		self.cursor = cursor;
	}
	
	fn undo_replace_at_with(&mut self, cursor: Cursor, old_data: &[u8]) {
		self.contents.splice(
			cursor.range(),
			old_data.iter().copied()
		);
		
		self.cursor = cursor;
	}
}
