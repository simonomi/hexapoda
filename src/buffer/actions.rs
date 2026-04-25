use std::{cmp::min, collections::hash_set::Entry, convert::identity, fs::File, io::Write, iter, mem::{replace, swap}};
use itertools::Itertools;
use ratatui::{style::{Color, Stylize}, text::Span};
use crate::{BYTES_OF_PADDING, BYTES_PER_LINE, LINES_OF_PADDING, action::BufferAction, buffer::{Buffer, InspectionStatus, Mode, PartialAction}, cursor::Cursor, edit_action::EditAction, popup::Popup, window_size::WindowSize};

impl Buffer {
	pub fn execute(&mut self, action: BufferAction, window_size: WindowSize) {
		match action {
			BufferAction::NormalMode => self.normal_mode(),
			BufferAction::SelectMode => self.select_mode(),
			
			BufferAction::Goto => self.goto(),
			BufferAction::View => self.view(),
			BufferAction::Replace => self.replace(),
			BufferAction::Space => self.space(),
			BufferAction::Repeat => self.repeat(),
			BufferAction::To => self.to(),
			
			BufferAction::ScrollDown => self.scroll_down(window_size),
			BufferAction::ScrollUp => self.scroll_up(window_size),
			
			BufferAction::PageCursorHalfDown => self.page_cursor_half_down(window_size),
			BufferAction::PageCursorHalfUp => self.page_cursor_half_up(window_size),
			
			BufferAction::PageDown => self.page_down(window_size),
			BufferAction::PageUp => self.page_up(window_size),
			
			BufferAction::CollapseSelection => self.collapse_selection(),
			BufferAction::FlipSelections => self.flip_selection(window_size),
			
			BufferAction::Delete => self.delete(window_size),
			
			BufferAction::Undo => self.undo(window_size),
			BufferAction::Redo => self.redo(window_size),
			
			BufferAction::Save => self.save(),
			
			BufferAction::CopySelectionOnNextLine => self.copy_selection_on_next_line(window_size),
			
			BufferAction::RotateSelectionsBackward => self.rotate_selections_backward(window_size),
			BufferAction::RotateSelectionsForward => self.rotate_selections_forward(window_size),
			
			BufferAction::KeepPrimarySelection => self.keep_primary_selection(),
			BufferAction::RemovePrimarySelection => self.remove_primary_selection(),
			
			BufferAction::SplitSelectionsInto1s => self.split_selections_into_size(1, window_size),
			BufferAction::SplitSelectionsInto2s => self.split_selections_into_size(2, window_size),
			BufferAction::SplitSelectionsInto3s => self.split_selections_into_size(3, window_size),
			BufferAction::SplitSelectionsInto4s => self.split_selections_into_size(4, window_size),
			BufferAction::SplitSelectionsInto5s => self.split_selections_into_size(5, window_size),
			BufferAction::SplitSelectionsInto6s => self.split_selections_into_size(6, window_size),
			BufferAction::SplitSelectionsInto7s => self.split_selections_into_size(7, window_size),
			BufferAction::SplitSelectionsInto8s => self.split_selections_into_size(8, window_size),
			BufferAction::SplitSelectionsInto9s => self.split_selections_into_size(9, window_size),
			
			BufferAction::JumpToSelectedOffset => self.jump_to_selected_offset(window_size),
			BufferAction::JumpToSelectedOffsetRelativeToMark => self.jump_to_selected_offset_relative_to_mark(window_size),
			
			BufferAction::ToggleMark => self.toggle_mark(),
			
			BufferAction::AlignViewCenter => self.align_view_center(window_size),
			BufferAction::AlignViewBottom => self.align_view_bottom(window_size),
			BufferAction::AlignViewTop => self.align_view_top(),
			
			BufferAction::ExtendToMark => self.extend_to_mark(window_size),
			BufferAction::ExtendToNull => self.extend_to_null(window_size),
			BufferAction::ExtendToFF => self.extend_to_FF(window_size),
			
			BufferAction::InspectSelection => self.inspect_selection(),
			BufferAction::InspectSelectionColor => self.inspect_selection_color(),
		}
	}
	
	const fn normal_mode(&mut self) {
		self.mode = Mode::Normal;
	}
	
	const fn select_mode(&mut self) {
		self.mode = Mode::Select;
	}
	
	const fn goto(&mut self) {
		self.partial_action = Some(PartialAction::Goto);
	}
	
	const fn view(&mut self) {
		self.partial_action = Some(PartialAction::View);
	}
	
	const fn replace(&mut self) {
		if !self.contents.is_empty() {
			self.partial_action = Some(PartialAction::Replace);
		}
	}
	
	const fn space(&mut self) {
		self.partial_action = Some(PartialAction::Space);
	}
	
	const fn repeat(&mut self) {
		self.partial_action = Some(PartialAction::Repeat);
	}
	
	const fn to(&mut self) {
		self.partial_action = Some(PartialAction::To);
	}
	
	pub fn scroll_down(&mut self, window_size: WindowSize) {
		if self.contents.len() <= BYTES_OF_PADDING { return; }
		
		self.scroll_position = min(
			self.scroll_position + BYTES_PER_LINE,
			self.contents.len() - BYTES_OF_PADDING - self.contents.len() % BYTES_PER_LINE
		);
		
		if window_size.hex_rows() > LINES_OF_PADDING * 2 {
			self.primary_cursor.clamp(
				self.scroll_position + BYTES_OF_PADDING,
				window_size.visible_byte_count() - BYTES_OF_PADDING * 2
			);
		} else {
			self.primary_cursor.clamp(self.scroll_position, window_size.visible_byte_count());
		}
		
		self.combine_cursors_if_overlapping();
	}
	
	pub fn scroll_up(&mut self, window_size: WindowSize) {
		self.scroll_position = self.scroll_position.saturating_sub(BYTES_PER_LINE);
		
		if window_size.hex_rows() > LINES_OF_PADDING * 2 {
			self.primary_cursor.clamp(
				self.scroll_position + BYTES_OF_PADDING,
				window_size.visible_byte_count() - BYTES_OF_PADDING * 2
			);
		} else {
			self.primary_cursor.clamp(self.scroll_position, window_size.visible_byte_count());
		}
		
		self.combine_cursors_if_overlapping();
	}
	
	fn page_cursor_half_down(&mut self, window_size: WindowSize) {
		if self.contents.len() <= BYTES_OF_PADDING { return; }
		
		let old_scroll_position = self.scroll_position;
		
		self.scroll_position = min(
			self.scroll_position + (window_size.visible_byte_count() / 2).next_multiple_of(BYTES_PER_LINE),
			self.contents.len() - BYTES_OF_PADDING - self.contents.len() % BYTES_PER_LINE
		);
		
		let scroll_position_change = self.scroll_position - old_scroll_position;
		let max_contents_index = self.max_contents_index();
		
		self.primary_cursor.head = min(
			self.primary_cursor.head + scroll_position_change,
			max_contents_index
		);
		self.primary_cursor.tail = min(
			self.primary_cursor.tail + scroll_position_change,
			max_contents_index
		);
		
		for cursor in &mut self.cursors {
			cursor.head = (cursor.head + scroll_position_change).min(max_contents_index);
			cursor.tail = (cursor.tail + scroll_position_change).min(max_contents_index);
		}
		
		self.combine_cursors_if_overlapping();
	}
	
	fn page_cursor_half_up(&mut self, window_size: WindowSize) {
		let old_scroll_position = self.scroll_position;
		
		self.scroll_position = self.scroll_position.saturating_sub(
			(window_size.visible_byte_count() / 2).next_multiple_of(BYTES_PER_LINE)
		);
		
		let scroll_position_change = old_scroll_position - self.scroll_position;
		let max_contents_index = self.max_contents_index();
		
		self.primary_cursor.head = min(
			self.primary_cursor.head - scroll_position_change,
			max_contents_index
		);
		self.primary_cursor.tail = min(
			self.primary_cursor.tail - scroll_position_change,
			max_contents_index
		);
		
		for cursor in &mut self.cursors {
			cursor.head = (cursor.head - scroll_position_change).min(max_contents_index);
			cursor.tail = (cursor.tail - scroll_position_change).min(max_contents_index);
		}
		
		self.combine_cursors_if_overlapping();
	}
	
	fn page_down(&mut self, window_size: WindowSize) {
		if self.contents.len() <= BYTES_OF_PADDING { return; }
		
		self.scroll_position = min(
			self.scroll_position + window_size.visible_byte_count(),
			self.max_contents_index() - BYTES_OF_PADDING - self.max_contents_index() % BYTES_PER_LINE
		);
		
		if window_size.hex_rows() > LINES_OF_PADDING * 2 {
			self.primary_cursor.clamp(
				self.scroll_position + BYTES_OF_PADDING,
				window_size.visible_byte_count() - BYTES_OF_PADDING * 2
			);
		} else {
			self.primary_cursor.clamp(self.scroll_position, window_size.visible_byte_count());
		}
		
		self.combine_cursors_if_overlapping();
	}
	
	fn page_up(&mut self, window_size: WindowSize) {
		self.scroll_position = self.scroll_position.saturating_sub(
			window_size.visible_byte_count()
		);
		
		if window_size.hex_rows() > LINES_OF_PADDING * 2 {
			self.primary_cursor.clamp(
				self.scroll_position + BYTES_OF_PADDING,
				window_size.visible_byte_count() - BYTES_OF_PADDING * 2
			);
		} else {
			self.primary_cursor.clamp(self.scroll_position, window_size.visible_byte_count());
		}
		
		self.combine_cursors_if_overlapping();
	}
	
	fn collapse_selection(&mut self) {
		self.primary_cursor.collapse();
		
		for cursor in &mut self.cursors {
			cursor.collapse();
		}
	}
	
	fn flip_selection(&mut self, window_size: WindowSize) {
		self.primary_cursor.flip();
		
		for cursor in &mut self.cursors {
			cursor.flip();
		}
		
		self.clamp_screen_to_primary_cursor(window_size);
	}
	
	fn delete(&mut self, window_size: WindowSize) {
		if !self.contents.is_empty() {
			self.execute_and_add(
				EditAction::Delete {
					primary_cursor: self.primary_cursor,
					cursors: self.cursors.clone(),
					primary_old_data: self.contents[self.primary_cursor.range()].into(),
					old_data: self.cursors
						.iter()
						.map(|cursor| self.contents[cursor.range()].to_vec())
						.collect(),
				},
				window_size
			);
		}
		
		if self.mode == Mode::Select {
			self.mode = Mode::Normal;
		}
	}
	
	fn undo(&mut self, window_size: WindowSize) {
		if self.time_traveling == Some(0) || self.edit_history.is_empty() { return; }
		
		let current_date = self.time_traveling
			.map_or(self.edit_history.len() - 1, |date| date - 1);
		
		self.time_traveling = Some(current_date);
		
		let edit_action = replace(
			&mut self.edit_history[current_date],
			EditAction::Placeholder
		);
		
		self.undo_edit(&edit_action, window_size);
		
		self.edit_history[current_date] = edit_action;
	}
	
	fn redo(&mut self, window_size: WindowSize) {
		let Some(previous_date) = self.time_traveling else { return; };
		
		let current_date = previous_date + 1;
		
		self.time_traveling = if current_date == self.edit_history.len() {
			None
		} else {
			Some(current_date)
		};
		
		let edit_action = replace(
			&mut self.edit_history[previous_date],
			EditAction::Placeholder
		);
		
		self.execute_edit(&edit_action, window_size);
		
		self.edit_history[previous_date] = edit_action;
	}
	
	fn save(&mut self) {
		let mut file = File::create(&self.file_path).unwrap();
		file.write_all(&self.contents).unwrap();
		
		self.last_saved_at = Some(
			self.time_traveling.unwrap_or(self.edit_history.len())
		);
	}
	
	fn copy_selection_on_next_line(&mut self, window_size: WindowSize) {
		let new_cursors: Vec<Cursor> = iter::once(&self.primary_cursor)
			.chain(&self.cursors)
			.filter_map(|cursor| {
				let number_of_lines_tall = (cursor.upper_bound() - cursor.lower_bound()) / BYTES_PER_LINE;
				let offset_to_add = (number_of_lines_tall + 1) * BYTES_PER_LINE;
				
				if cursor.lower_bound() + offset_to_add < self.contents.len() {
					Some(
						Cursor {
							head: min(cursor.head + offset_to_add, self.max_contents_index()),
							tail: min(cursor.tail + offset_to_add, self.max_contents_index())
						}
					)
				} else {
					None
				}
			})
			.collect();
		
		self.cursors.extend(new_cursors);
		self.cursors.sort_by_key(|cursor| cursor.head);
		
		self.combine_cursors_if_overlapping();
		self.rotate_selections_forward(window_size);
	}
	
	fn rotate_selections_backward(&mut self, window_size: WindowSize) {
		if self.cursors.is_empty() { return; }
		
		let next_cursor_index = self.cursors
			.binary_search_by_key(&self.primary_cursor.head, |cursor| cursor.head)
			.unwrap_or_else(identity);
		
		
		if next_cursor_index == 0 {
			let cursor_count = self.cursors.len();
			swap(&mut self.primary_cursor, &mut self.cursors[cursor_count - 1]);
			
			self.cursors.sort_by_key(|cursor| cursor.head);
		} else {
			swap(&mut self.primary_cursor, &mut self.cursors[next_cursor_index - 1]);
		}
		
		self.clamp_screen_to_primary_cursor(window_size);
	}
	
	fn rotate_selections_forward(&mut self, window_size: WindowSize) {
		if self.cursors.is_empty() { return; }
		
		let next_cursor_index = self.cursors
			.binary_search_by_key(&self.primary_cursor.head, |cursor| cursor.head)
			.unwrap_or_else(identity);
		
		if next_cursor_index == self.cursors.len() {
			swap(&mut self.primary_cursor, &mut self.cursors[0]);
			
			// TODO: is a full sort necessary ?
			self.cursors.sort_by_key(|cursor| cursor.head);
		} else {
			swap(&mut self.primary_cursor, &mut self.cursors[next_cursor_index]);
		}
		
		self.clamp_screen_to_primary_cursor(window_size);
	}
	
	fn keep_primary_selection(&mut self) {
		self.cursors.clear();
	}
	
	fn remove_primary_selection(&mut self) {
		if self.cursors.is_empty() { return; }
		
		let next_cursor_index = self.cursors
			.binary_search_by_key(&self.primary_cursor.head, |cursor| cursor.head)
			.unwrap_or_else(identity);
		
		if next_cursor_index == self.cursors.len() {
			self.primary_cursor = self.cursors.remove(0);
		} else {
			self.primary_cursor = self.cursors.remove(next_cursor_index);
		}
	}
	
	fn split_selections_into_size(&mut self, size: usize, window_size: WindowSize) {
		if !iter::once(&self.primary_cursor)
			.chain(&self.cursors)
			.all(|cursor| cursor.len().is_multiple_of(size))
		{
			self.alert_message = Span::from(
				format!("not all selections are a multiple of {size} long")
			).red();
			return;
		}
		
		let mut new_cursors = iter::once(self.primary_cursor)
			.chain(self.cursors.iter().copied())
			.flat_map(|cursor| {
				cursor
					.range()
					.step_by(size)
					.map(|tail| Cursor { head: tail + size - 1, tail })
			});
		
		self.primary_cursor = new_cursors.next().unwrap();
		self.cursors = new_cursors
			.sorted_by_key(|cursor| cursor.head)
			.collect();
		
		self.clamp_screen_to_primary_cursor(window_size);
	}
	
	fn jump_to_selected_offset(&mut self, window_size: WindowSize) {
		if !iter::once(&self.primary_cursor)
			.chain(&self.cursors)
			.all(|cursor| {
				bytes_to_nat(&self.contents[cursor.range()])
					.map(|nat| nat as usize)
					.is_some_and(|offset| offset < self.contents.len())
			})
		{
			if self.cursors.is_empty() {
				self.alert_message = Span::from(
					"selection is not a valid offset"
				).red();
			} else {
				self.alert_message = Span::from(
					"not all selections are valid offsets"
				).red();
			}
			return;
		}
		
		self.primary_cursor = Cursor::at(
			bytes_to_nat(&self.contents[self.primary_cursor.range()]).unwrap() as usize
		);
		
		for cursor in &mut self.cursors {
			*cursor = Cursor::at(
				bytes_to_nat(&self.contents[cursor.range()]).unwrap() as usize
			);
		}
		
		self.cursors.sort_by_key(|cursor| cursor.head);
		
		self.combine_cursors_if_overlapping();
		self.clamp_screen_to_primary_cursor(window_size);
	}
	
	fn jump_to_selected_offset_relative_to_mark(&mut self, window_size: WindowSize) {
		let mut sorted_marks: Vec<_> = self.marks.iter().copied().collect();
		sorted_marks.sort_unstable();
		
		if !iter::once(&self.primary_cursor)
			.chain(&self.cursors)
			.all(|cursor| {
				bytes_to_nat(&self.contents[cursor.range()])
					.map(|offset| mark_before(cursor.lower_bound(), &sorted_marks) + offset as usize)
					.is_some_and(|offset| offset < self.contents.len())
			})
		{
			if self.cursors.is_empty() {
				self.alert_message = Span::from(
					"selection is not a valid offset"
				).red();
			} else {
				self.alert_message = Span::from(
					"not all selections are valid offsets"
				).red();
			}
			return;
		}
		
		self.primary_cursor = Cursor::at(
			bytes_to_nat(&self.contents[self.primary_cursor.range()])
				.map(|offset| {
					mark_before(self.primary_cursor.lower_bound(), &sorted_marks) + offset as usize
				})
				.unwrap()
		);
		
		for cursor in &mut self.cursors {
			*cursor = Cursor::at(
				bytes_to_nat(&self.contents[cursor.range()])
				.map(|offset| {
					mark_before(cursor.lower_bound(), &sorted_marks) + offset as usize
				})
				.unwrap()
			);
		}
		
		self.cursors.sort_by_key(|cursor| cursor.head);
		
		self.combine_cursors_if_overlapping();
		self.clamp_screen_to_primary_cursor(window_size);
	}
	
	fn toggle_mark(&mut self) {
		match self.marks.entry(self.primary_cursor.lower_bound()) {
			Entry::Occupied(occupied_entry) => { occupied_entry.remove(); },
			Entry::Vacant(vacant_entry) => vacant_entry.insert(),
		}
		
		for cursor in &self.cursors {
			match self.marks.entry(cursor.lower_bound()) {
				Entry::Occupied(occupied_entry) => { occupied_entry.remove(); },
				Entry::Vacant(vacant_entry) => vacant_entry.insert(),
			}
		}
	}
	
	const fn align_view_center(&mut self, window_size: WindowSize) {
		let half_a_screen = window_size.visible_byte_count() / 2;
		
		self.scroll_position = self.primary_cursor.head
			.saturating_sub(self.primary_cursor.head % BYTES_PER_LINE)
			.saturating_sub(half_a_screen - (half_a_screen % BYTES_PER_LINE));
	}
	
	fn align_view_bottom(&mut self, window_size: WindowSize) {
		self.scroll_position = self.primary_cursor.head
			.saturating_sub(self.primary_cursor.head % BYTES_PER_LINE)
			.saturating_sub(
				window_size
					.visible_byte_count()
					.saturating_sub(BYTES_PER_LINE + BYTES_OF_PADDING)
			)
			.min(self.max_contents_index() - self.max_contents_index() % BYTES_PER_LINE);
	}
	
	const fn align_view_top(&mut self) {
		self.scroll_position = self.primary_cursor.head
			.saturating_sub(self.primary_cursor.head % BYTES_PER_LINE)
			.saturating_sub(BYTES_OF_PADDING);
	}
	
	fn extend_to_mark(&mut self, window_size: WindowSize) {
		let mut sorted_marks: Vec<_> = self.marks.iter().copied().collect();
		sorted_marks.sort_unstable();
		
		let max_contents_index = self.max_contents_index();
		
		let mark_after_primary = mark_after(
			self.primary_cursor.head,
			&sorted_marks,
			max_contents_index
		);
		
		self.primary_cursor.tail = self.primary_cursor.head;
		self.primary_cursor.head = mark_after_primary - 1;
		
		for cursor in &mut self.cursors {
			let mark_after_cursor = mark_after(
				cursor.head,
				&sorted_marks,
				max_contents_index
			);
			
			cursor.tail = cursor.head;
			cursor.head = mark_after_cursor - 1;
		}
		
		self.combine_cursors_if_overlapping();
		self.clamp_screen_to_primary_cursor(window_size);
	}
	
	fn extend_to_null(&mut self, window_size: WindowSize) {
		if let Some(null_offset_after_primary) = self.contents[self.primary_cursor.head..]
			.iter()
			.skip(1)
			.position(|&byte| byte == 0)
		{
			self.primary_cursor.tail = self.primary_cursor.head;
			self.primary_cursor.head += null_offset_after_primary;
		}
		
		for cursor in &mut self.cursors {
			if let Some(null_offset_after_primary) = self.contents[cursor.head..]
				.iter()
				.skip(1)
				.position(|&byte| byte == 0)
			{
				cursor.tail = cursor.head;
				cursor.head += null_offset_after_primary;
			}
		}
		
		self.combine_cursors_if_overlapping();
		self.clamp_screen_to_primary_cursor(window_size);
	}
	
	#[allow(non_snake_case)]
	fn extend_to_FF(&mut self, window_size: WindowSize) {
		if let Some(null_offset_after_primary) = self.contents[self.primary_cursor.head..]
			.iter()
			.skip(1)
			.position(|&byte| byte == 0xFF)
		{
			self.primary_cursor.tail = self.primary_cursor.head;
			self.primary_cursor.head += null_offset_after_primary;
		}
		
		for cursor in &mut self.cursors {
			if let Some(null_offset_after_primary) = self.contents[cursor.head..]
				.iter()
				.skip(1)
				.position(|&byte| byte == 0xFF)
			{
				cursor.tail = cursor.head;
				cursor.head += null_offset_after_primary;
			}
		}
		
		self.combine_cursors_if_overlapping();
		self.clamp_screen_to_primary_cursor(window_size);
	}
	
	#[allow(clippy::too_many_lines)]
	fn inspect_selection(&mut self) {
		if self.inspection_status == Some(InspectionStatus::Normal) {
			self.inspection_status = None;
			return;
		}
		
		self.inspection_status = Some(InspectionStatus::Normal);
		
		self.popups.extend(
			iter::once(&self.primary_cursor)
				.chain(&self.cursors)
				.filter_map(|cursor| {
					let selection = &self.contents[cursor.range()];
					
					let popup_lines = inspect(selection);
					
					if popup_lines.is_empty() {
						None
					} else {
						Some(Popup::new(cursor.lower_bound(), popup_lines))
					}
				})
				.sorted_unstable_by_key(|popup| popup.at)
		);
		
		if self.popups.is_empty() {
			self.inspection_status = None;
		}
	}
	
	fn inspect_selection_color(&mut self) {
		if self.inspection_status == Some(InspectionStatus::ColorsOnly) {
			self.inspection_status = None;
			return;
		}
		
		self.inspection_status = Some(InspectionStatus::ColorsOnly);
		
		self.popups.extend(
			iter::once(&self.primary_cursor)
				.chain(&self.cursors)
				.filter_map(|cursor| {
					let selection = &self.contents[cursor.range()];
					
					let popup_lines = inspect_color(selection);
					
					if popup_lines.is_empty() {
						None
					} else {
						Some(Popup::new(cursor.lower_bound(), popup_lines))
					}
				})
				.sorted_unstable_by_key(|popup| popup.at)
		);
		
		if self.popups.is_empty() {
			self.inspection_status = None;
		}
	}
}

fn inspect(selection: &[u8]) -> Vec<Span<'static>> {
	let nat = bytes_to_nat(selection);
	
	let int = nat.and_then(|nat| nat_to_int_if_different(nat, selection.len()));
	
	let binary = nat
		.filter(|_| selection.len() == 1)
		.map(|nat| {
			let lower_bits = nat & 0b1111;
			let upper_bits = nat >> 4;
			
			format!("{upper_bits:04b}_{lower_bits:04b}").into()
		});
	
	let utf8 = str::from_utf8(selection).ok()
		.filter(|_| selection.len() != 1)
		.map(|utf8| utf8.trim_suffix('\0'))
		.filter(|utf8| !utf8.contains(is_illegal_control_character))
		.map(|utf8| Span::from(format!("\"{utf8}\"")).red());
	
	let fixedpoint2012 = nat
		.filter(|_| selection.len() == 4)
		.map(|nat| f64::from(nat as u32) / f64::from(1 << 12))
		.map(|fixedpoint2012| {
			let two_decimals_is_enough = (fixedpoint2012 * 100.0).fract() == 0.0;
			let approximate_symbol = if two_decimals_is_enough { "" } else { "~" };
			
			format!("20.12: {approximate_symbol}{fixedpoint2012:.2}").into()
		});
	
	let fixedpoint2012_signed = int
		.filter(|_| selection.len() == 4)
		.map(|int| f64::from(int as i32) / f64::from(1 << 12))
		.map(|fixedpoint2012_signed| {
			let two_decimals_is_enough = (fixedpoint2012_signed * 100.0).fract() == 0.0;
			let approximate_symbol = if two_decimals_is_enough { "" } else { "~" };
			
			format!("i20.12: {approximate_symbol}{fixedpoint2012_signed:.2}").into()
		});
	
	let fixedpoint1616 = nat
		.filter(|_| selection.len() == 4)
		.map(|nat| f64::from(nat as u32) / f64::from(1 << 16))
		.map(|fixedpoint1616| {
			let two_decimals_is_enough = (fixedpoint1616 * 100.0).fract() == 0.0;
			let approximate_symbol = if two_decimals_is_enough { "" } else { "~" };
			
			format!("16.16: {approximate_symbol}{fixedpoint1616:.2}").into()
		});
	
	let fixedpoint1616_signed = int
		.filter(|_| selection.len() == 4)
		.map(|int| f64::from(int as i32) / f64::from(1 << 16))
		.map(|fixedpoint1616_signed| {
			let two_decimals_is_enough = (fixedpoint1616_signed * 100.0).fract() == 0.0;
			let approximate_symbol = if two_decimals_is_enough { "" } else { "~" };
			
			format!("i16.16: {approximate_symbol}{fixedpoint1616_signed:.2}").into()
		});
	
	let fixedpoint124 = nat
		.filter(|_| selection.len() == 2)
		.map(|nat| f64::from(nat as u16) / f64::from(1 << 4))
		.map(|fixedpoint124| {
			let two_decimals_is_enough = (fixedpoint124 * 100.0).fract() == 0.0;
			let approximate_symbol = if two_decimals_is_enough { "" } else { "~" };
			
			format!("12.4: {approximate_symbol}{fixedpoint124:.2}").into()
		});
	
	let fixedpoint88 = nat
		.filter(|_| selection.len() == 2)
		.map(|nat| f64::from(nat as u16) / f64::from(1 << 8))
		.map(|fixedpoint88| {
			let two_decimals_is_enough = (fixedpoint88 * 100.0).fract() == 0.0;
			let approximate_symbol = if two_decimals_is_enough { "" } else { "~" };
			
			format!("8.8: {approximate_symbol}{fixedpoint88:.2}").into()
		});
	
	let fixedpoint412 = nat
		.filter(|_| selection.len() == 2)
		.map(|nat| f64::from(nat as u16) / f64::from(1 << 12))
		.map(|fixedpoint412| {
			let two_decimals_is_enough = (fixedpoint412 * 100.0).fract() == 0.0;
			let approximate_symbol = if two_decimals_is_enough { "" } else { "~" };
			
			format!("4.12: {approximate_symbol}{fixedpoint412:.2}").into()
		});
	
	let color888 = (selection.len() == 3)
		.then(|| [selection[0], selection[1], selection[2]])
		.map(|[red, green, blue]| {
			Span::from(format!("#{red:02X}{green:02X}{blue:02X}"))
				.fg(Color::Rgb(red, green, blue))
			
		});
	
	let color555 = nat
		.filter(|_| selection.len() == 2)
		.filter(|&nat| nat >> 15 == 0)
		.map(|nat| color555_to_color888(nat as u16))
		.map(|[red, green, blue]| {
			Span::from(format!("555: #{red:02X}{green:02X}{blue:02X}"))
				.fg(Color::Rgb(red, green, blue))
			
		});
	
	int.map(|int| format!("{int}").into())
		.into_iter()
		.chain(nat.map(|nat| format!("{nat}").into()))
		.chain(binary)
		.chain(utf8)
		.chain(fixedpoint2012_signed)
		.chain(fixedpoint2012)
		.chain(fixedpoint1616_signed)
		.chain(fixedpoint1616)
		.chain(fixedpoint124)
		.chain(fixedpoint88)
		.chain(fixedpoint412)
		.chain(color888)
		.chain(color555)
		.collect()
}

fn inspect_color(selection: &[u8]) -> Vec<Span<'static>> {
	let nat = bytes_to_nat(selection);
	
	let color888 = (selection.len() == 3)
		.then(|| [selection[0], selection[1], selection[2]])
		.map(|[red, green, blue]| {
			Span::from(format!("#{red:02X}{green:02X}{blue:02X}"))
				.fg(Color::Rgb(red, green, blue))
			
		});
	
	let color555 = nat
		.filter(|_| selection.len() == 2)
		.filter(|&nat| nat >> 15 == 0)
		.map(|nat| color555_to_color888(nat as u16))
		.map(|[red, green, blue]| {
			Span::from(format!("#{red:02X}{green:02X}{blue:02X}"))
				.fg(Color::Rgb(red, green, blue))
			
		});
	
	color888
		.into_iter()
		.chain(color555)
		.collect()
}

// MARK: helpers
impl Buffer {
	pub fn clamp_screen_to_primary_cursor(&mut self, window_size: WindowSize) {
		if self.primary_cursor.head < self.scroll_position + BYTES_OF_PADDING {
			self.align_view_top();
		} else if self.primary_cursor.head > self.scroll_position + (window_size.visible_byte_count() - 1).saturating_sub(BYTES_OF_PADDING) {
			self.align_view_bottom(window_size);
		}
	}
}

pub fn bytes_to_nat(bytes: &[u8]) -> Option<u64> {
	bytes
		.iter()
		.rev() // little-endian
		.skip_while(|&&byte| byte == 0)
		.try_fold(u64::default(), |result, &byte| {
			Some(result.shl_exact(8)? | u64::from(byte))
		})
}

const fn nat_to_int_if_different(nat: u64, bytes: usize) -> Option<i64> {
	match bytes {
		1 if nat >  i8::MAX as u64 => Some((nat as u8).cast_signed() as i64),
		2 if nat > i16::MAX as u64 => Some((nat as u16).cast_signed() as i64),
		4 if nat > i32::MAX as u64 => Some((nat as u32).cast_signed() as i64),
		8 if nat > i64::MAX as u64 => Some(nat.cast_signed()),
		_ => None,
	}
}

#[test]
fn nat_to_int_tests() {
	assert_eq!(nat_to_int_if_different(0, 1), None);
	assert_eq!(nat_to_int_if_different(i8::MAX as u64,     1), None);
	assert_eq!(nat_to_int_if_different(i8::MAX as u64 + 1, 1), Some(i8::MIN.into()));
	assert_eq!(nat_to_int_if_different(u8::MAX.into(),     1), Some(-1));
	
	assert_eq!(nat_to_int_if_different(0, 2), None);
	assert_eq!(nat_to_int_if_different(i16::MAX as u64,     2), None);
	assert_eq!(nat_to_int_if_different(i16::MAX as u64 + 1, 2), Some(i16::MIN.into()));
	assert_eq!(nat_to_int_if_different(u16::MAX.into(),     2), Some(-1));
}

// or 0 if no mark is before
fn mark_before(offset: usize, sorted_marks: &[usize]) -> usize {
	match sorted_marks.binary_search(&offset) {
		Ok(_) => offset,
		Err(0) => 0,
		Err(mark_after_index) => sorted_marks[mark_after_index - 1],
	}
}

// or end index if no mark is after
fn mark_after(offset: usize, sorted_marks: &[usize], max: usize) -> usize {
	if sorted_marks.is_empty() { return max + 1; }
	
	match sorted_marks.binary_search(&offset) {
		Ok(mark_before_index) => if mark_before_index == sorted_marks.len() - 1 {
			max + 1
		} else {
			sorted_marks[mark_before_index + 1]
		},
		Err(mark_after_index) => {
			if mark_after_index == sorted_marks.len() {
				max + 1
			} else {
				sorted_marks[mark_after_index]
			}
		},
	}
}

const fn is_illegal_control_character(character: char) -> bool {
	match character {
		'\t' | '\n' | '\r' => false,
		_ if character.is_ascii_control() => true,
		_ => false,
	}
}

const fn color555_to_color888(color555: u16) -> [u8; 3] {
	[
		// 8 is the ratio between the number of colors in 555 vs 888 (32:256)
		(color555       & 0b11111) as u8 * 8,
		(color555 >>  5 & 0b11111) as u8 * 8,
		(color555 >> 10 & 0b11111) as u8 * 8
	]
}
