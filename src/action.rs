use std::{cmp::min, collections::hash_set::Entry, convert::identity, fs::File, io::Write, iter, mem::{replace, swap}};
use itertools::Itertools;
use ratatui::{style::{Color, Stylize}, text::Span};
use serde::{Deserialize, Serialize};
use crate::{BYTES_OF_PADDING, BYTES_PER_LINE, LINES_OF_PADDING, app::WindowSize, buffer::{Buffer, InspectionStatus, Mode, PartialAction, Popup}, cursor::Cursor, edit_action::EditAction};

#[derive(Clone, Copy, Serialize, Deserialize)]
#[derive(Debug)]
#[serde(into = "&str")]
#[serde(try_from = "&str")]
pub enum Action {
	App(AppAction),
	Buffer(BufferAction),
	Cursor(CursorAction),
}

impl Action {
	pub const fn clears_popups(self) -> bool {
		use Action::*;
		
		match self {
			App(app_action) => app_action.clears_popups(),
			Buffer(buffer_action) => buffer_action.clears_popups(),
			Cursor(cursor_action) => cursor_action.clears_popups(),
		}
	}
	
	pub const fn is_inspection(self) -> bool {
		use Action::*;
		use BufferAction::*;
		
		matches!(self, Buffer(InspectSelection | InspectSelectionColor))
	}
}

impl From<Action> for &str {
	fn from(action: Action) -> Self {
		match action {
			Action::App(app_action) => app_action.into(),
			Action::Buffer(buffer_action) => buffer_action.into(),
			Action::Cursor(cursor_action) => cursor_action.into(),
		}
	}
}

impl TryFrom<&str> for Action {
	type Error = String;
	
	fn try_from(string: &str) -> Result<Self, String> {
		AppAction::try_from(string).map(Self::from)
			.or_else(|()| BufferAction::try_from(string).map(Self::from))
			.or_else(|()| CursorAction::try_from(string).map(Self::from))
			.map_err(|()| format!("invalid action: {string}"))
	}
}

// actions that act on the app as a whole, not just one buffer
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum AppAction {
	QuitIfSaved,
	Quit,
	
	PreviousBuffer,
	NextBuffer,
	
	Yank,
}

impl AppAction {
	const fn clears_popups(self) -> bool {
		use AppAction::*;
		
		#[allow(clippy::match_same_arms)]
		match self {
			QuitIfSaved => true,
			Quit => true,
			
			PreviousBuffer => false,
			NextBuffer => false,
			
			Yank => false,
		}
	}
}

impl From<AppAction> for &str {
	fn from(app_action: AppAction) -> Self {
		use AppAction::*;
		
		match app_action {
			QuitIfSaved => "quit_if_saved",
			Quit => "quit",
			
			PreviousBuffer => "previous_buffer",
			NextBuffer => "next_buffer",
			
			Yank => "yank",
		}
	}
}

impl From<AppAction> for Action {
	fn from(app_action: AppAction) -> Self {
		Self::App(app_action)
	}
}

impl TryFrom<&str> for AppAction {
	type Error = ();
	
	fn try_from(string: &str) -> Result<Self, ()> {
		use AppAction::*;
		
		match string {
			"quit_if_saved" => Ok(QuitIfSaved),
			"quit" => Ok(Quit),
			
			"previous_buffer" => Ok(PreviousBuffer),
			"next_buffer" => Ok(NextBuffer),
			
			"yank" => Ok(Yank),
			
			_ => Err(()),
		}
	}
}

#[derive(Clone, Copy, Deserialize)]
#[derive(Debug)]
pub enum BufferAction {
	NormalMode,
	SelectMode,
	
	Goto,
	View,
	Replace,
	Space,
	Repeat,
	To,
	
	ScrollDown,
	ScrollUp,
	
	PageCursorHalfDown,
	PageCursorHalfUp,
	
	PageDown,
	PageUp,
	
	CollapseSelection,
	FlipSelections,
	
	Delete,
	
	Undo,
	Redo,
	
	Save,
	
	CopySelectionOnNextLine,
	
	RotateSelectionsBackward,
	RotateSelectionsForward,
	
	KeepPrimarySelection,
	RemovePrimarySelection,
	
	SplitSelectionsInto1s,
	SplitSelectionsInto2s,
	SplitSelectionsInto3s,
	SplitSelectionsInto4s,
	SplitSelectionsInto5s,
	SplitSelectionsInto6s,
	SplitSelectionsInto7s,
	SplitSelectionsInto8s,
	SplitSelectionsInto9s,
	
	JumpToSelectedOffset,
	JumpToSelectedOffsetRelativeToMark,
	
	ToggleMark,
	
	AlignViewCenter,
	AlignViewBottom,
	AlignViewTop,
	
	ExtendToMark,
	ExtendToNull,
	ExtendToFF,
	
	InspectSelection,
	InspectSelectionColor,
}

impl BufferAction {
	const fn clears_popups(self) -> bool {
		use BufferAction::*;
		
		#[allow(clippy::match_same_arms)]
		match self {
			NormalMode => false,
			SelectMode => false,
			
			Goto => false,
			View => false,
			Replace => true,
			Space => false,
			Repeat => true,
			To => false,
			
			ScrollDown => true,
			ScrollUp => true,
			
			PageCursorHalfDown => true,
			PageCursorHalfUp => true,
			
			PageDown => true,
			PageUp => true,
			
			CollapseSelection => true,
			FlipSelections => false,
			
			Delete => true,
			
			Undo => true,
			Redo => true,
			
			Save => false,
			
			CopySelectionOnNextLine => true,
			
			RotateSelectionsBackward => false,
			RotateSelectionsForward => false,
			
			KeepPrimarySelection => true,
			RemovePrimarySelection => true,
			
			SplitSelectionsInto1s => true,
			SplitSelectionsInto2s => true,
			SplitSelectionsInto3s => true,
			SplitSelectionsInto4s => true,
			SplitSelectionsInto5s => true,
			SplitSelectionsInto6s => true,
			SplitSelectionsInto7s => true,
			SplitSelectionsInto8s => true,
			SplitSelectionsInto9s => true,
			
			JumpToSelectedOffset => true,
			JumpToSelectedOffsetRelativeToMark => true,
			
			ToggleMark => false,
			
			AlignViewCenter => false,
			AlignViewBottom => false,
			AlignViewTop => false,
			
			ExtendToMark => true,
			ExtendToNull => true,
			ExtendToFF => true,
			
			InspectSelection => true,
			InspectSelectionColor => true,
		}
	}
}

impl From<BufferAction> for &str {
	fn from(buffer_action: BufferAction) -> Self {
		use BufferAction::*;
		
		match buffer_action {
			NormalMode => "normal_mode",
			SelectMode => "select_mode",
			
			Goto => "goto",
			View => "view",
			Replace => "replace",
			Space => "space",
			Repeat => "repeat",
			To => "to",
			
			ScrollDown => "scroll_down",
			ScrollUp => "scroll_up",
			
			PageCursorHalfDown => "page_cursor_half_down",
			PageCursorHalfUp => "page_cursor_half_up",
			
			PageDown => "page_down",
			PageUp => "page_up",
			
			CollapseSelection => "collapse_selection",
			FlipSelections => "flip_selections",
			
			Delete => "delete",
			
			Undo => "undo",
			Redo => "redo",
			
			Save => "save",
			
			CopySelectionOnNextLine => "copy_selection_on_next_line",
			
			RotateSelectionsBackward => "rotate_selections_backward",
			RotateSelectionsForward => "rotate_selections_forward",
			
			KeepPrimarySelection => "keep_primary_selection",
			RemovePrimarySelection => "remove_primary_selection",
			
			SplitSelectionsInto1s => "split_selections_into_1_s",
			SplitSelectionsInto2s => "split_selections_into_2_s",
			SplitSelectionsInto3s => "split_selections_into_3_s",
			SplitSelectionsInto4s => "split_selections_into_4_s",
			SplitSelectionsInto5s => "split_selections_into_5_s",
			SplitSelectionsInto6s => "split_selections_into_6_s",
			SplitSelectionsInto7s => "split_selections_into_7_s",
			SplitSelectionsInto8s => "split_selections_into_8_s",
			SplitSelectionsInto9s => "split_selections_into_9_s",
			
			JumpToSelectedOffset => "jump_to_selected_offset",
			JumpToSelectedOffsetRelativeToMark => "jump_to_selected_offset_relative_to_mark",
			
			ToggleMark => "toggle_mark",
			
			AlignViewCenter => "align_view_center",
			AlignViewBottom => "align_view_bottom",
			AlignViewTop => "align_view_top",
			
			ExtendToMark => "extend_to_mark",
			ExtendToNull => "extend_to_null",
			ExtendToFF => "extend_to_ff",
			
			InspectSelection => "inspect_selection",
			InspectSelectionColor => "inspect_selection_color",
		}
	}
}

impl From<BufferAction> for Action {
	fn from(buffer_action: BufferAction) -> Self {
		Self::Buffer(buffer_action)
	}
}

impl TryFrom<&str> for BufferAction {
	type Error = ();
	
	fn try_from(string: &str) -> Result<Self, ()> {
		use BufferAction::*;
		
		match string {
			"normal_mode" => Ok(NormalMode),
			"select_mode" => Ok(SelectMode),
			
			"goto" => Ok(Goto),
			"view" => Ok(View),
			"replace" => Ok(Replace),
			"space" => Ok(Space),
			"repeat" => Ok(Repeat),
			"to" => Ok(To),
			
			"scroll_down" => Ok(ScrollDown),
			"scroll_up" => Ok(ScrollUp),
			
			"page_cursor_half_down" => Ok(PageCursorHalfDown),
			"page_cursor_half_up" => Ok(PageCursorHalfUp),
			
			"page_down" => Ok(PageDown),
			"page_up" => Ok(PageUp),
			
			"collapse_selection" => Ok(CollapseSelection),
			"flip_selections" => Ok(FlipSelections),
			
			"delete" => Ok(Delete),
			
			"undo" => Ok(Undo),
			"redo" => Ok(Redo),
			
			"save" => Ok(Save),
			
			"copy_selection_on_next_line" => Ok(CopySelectionOnNextLine),
			
			"rotate_selections_backward" => Ok(RotateSelectionsBackward),
			"rotate_selections_forward" => Ok(RotateSelectionsForward),
			
			"keep_primary_selection" => Ok(KeepPrimarySelection),
			"remove_primary_selection" => Ok(RemovePrimarySelection),
			
			"split_selections_into_1_s" => Ok(SplitSelectionsInto1s),
			"split_selections_into_2_s" => Ok(SplitSelectionsInto2s),
			"split_selections_into_3_s" => Ok(SplitSelectionsInto3s),
			"split_selections_into_4_s" => Ok(SplitSelectionsInto4s),
			"split_selections_into_5_s" => Ok(SplitSelectionsInto5s),
			"split_selections_into_6_s" => Ok(SplitSelectionsInto6s),
			"split_selections_into_7_s" => Ok(SplitSelectionsInto7s),
			"split_selections_into_8_s" => Ok(SplitSelectionsInto8s),
			"split_selections_into_9_s" => Ok(SplitSelectionsInto9s),
			
			"jump_to_selected_offset" => Ok(JumpToSelectedOffset),
			"jump_to_selected_offset_relative_to_mark" => Ok(JumpToSelectedOffsetRelativeToMark),
			
			"toggle_mark" => Ok(ToggleMark),
			
			"align_view_center" => Ok(AlignViewCenter),
			"align_view_bottom" => Ok(AlignViewBottom),
			"align_view_top" => Ok(AlignViewTop),
			
			"extend_to_mark" => Ok(ExtendToMark),
			"extend_to_null" => Ok(ExtendToNull),
			"extend_to_ff" => Ok(ExtendToFF),
			
			"inspect_selection" => Ok(InspectSelection),
			"inspect_selection_color" => Ok(InspectSelectionColor),
			
			_ => Err(()),
		}
	}
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[derive(Debug)]
#[serde(rename_all = "snake_case")]
pub enum CursorAction {
	MoveByteUp,
	MoveByteDown,
	MoveByteLeft,
	MoveByteRight,
	
	ExtendByteUp,
	ExtendByteDown,
	ExtendByteLeft,
	ExtendByteRight,
	
	GotoLineStart,
	GotoLineEnd,
	GotoFileStart,
	GotoFileEnd,
	
	MoveNextWordStart,
	MoveNextWordEnd,
	MovePreviousWordStart,
	
	ExtendNextWordStart,
	ExtendNextWordEnd,
	ExtendPreviousWordStart,
	
	ExtendLineBelow,
	ExtendLineAbove,
}

impl CursorAction {
	const fn clears_popups(self) -> bool {
		use CursorAction::*;
		
		#[allow(clippy::match_same_arms)]
		match self {
			MoveByteUp => true,
			MoveByteDown => true,
			MoveByteLeft => true,
			MoveByteRight => true,
			
			ExtendByteUp => true,
			ExtendByteDown => true,
			ExtendByteLeft => true,
			ExtendByteRight => true,
			
			GotoLineStart => true,
			GotoLineEnd => true,
			GotoFileStart => true,
			GotoFileEnd => true,
			
			MoveNextWordStart => true,
			MoveNextWordEnd => true,
			MovePreviousWordStart => true,
			
			ExtendNextWordStart => true,
			ExtendNextWordEnd => true,
			ExtendPreviousWordStart => true,
			
			ExtendLineBelow => true,
			ExtendLineAbove => true,
		}
	}
}

impl From<CursorAction> for &str {
	fn from(cursor_action: CursorAction) -> Self {
		use CursorAction::*;
		
		match cursor_action {
			MoveByteUp => "move_byte_up",
			MoveByteDown => "move_byte_down",
			MoveByteLeft => "move_byte_left",
			MoveByteRight => "move_byte_right",
			
			ExtendByteUp => "extend_byte_up",
			ExtendByteDown => "extend_byte_down",
			ExtendByteLeft => "extend_byte_left",
			ExtendByteRight => "extend_byte_right",
			
			GotoLineStart => "goto_line_start",
			GotoLineEnd => "goto_line_end",
			GotoFileStart => "goto_file_start",
			GotoFileEnd => "goto_file_end",
			
			MoveNextWordStart => "move_next_word_start",
			MoveNextWordEnd => "move_next_word_end",
			MovePreviousWordStart => "move_previous_word_start",
			
			ExtendNextWordStart => "extend_next_word_start",
			ExtendNextWordEnd => "extend_next_word_end",
			ExtendPreviousWordStart => "extend_previous_word_start",
			
			ExtendLineBelow => "extend_line_below",
			ExtendLineAbove => "extend_line_above",
		}
	}
}

impl From<CursorAction> for Action {
	fn from(cursor_action: CursorAction) -> Self {
		Self::Cursor(cursor_action)
	}
}

impl TryFrom<&str> for CursorAction {
	type Error = ();
	
	fn try_from(string: &str) -> Result<Self, ()> {
		use CursorAction::*;
		
		match string {
			"move_byte_up" => Ok(MoveByteUp),
			"move_byte_down" => Ok(MoveByteDown),
			"move_byte_left" => Ok(MoveByteLeft),
			"move_byte_right" => Ok(MoveByteRight),
			
			"extend_byte_up" => Ok(ExtendByteUp),
			"extend_byte_down" => Ok(ExtendByteDown),
			"extend_byte_left" => Ok(ExtendByteLeft),
			"extend_byte_right" => Ok(ExtendByteRight),
			
			"goto_line_start" => Ok(GotoLineStart),
			"goto_line_end" => Ok(GotoLineEnd),
			"goto_file_start" => Ok(GotoFileStart),
			"goto_file_end" => Ok(GotoFileEnd),
			
			"move_next_word_start" => Ok(MoveNextWordStart),
			"move_next_word_end" => Ok(MoveNextWordEnd),
			"move_previous_word_start" => Ok(MovePreviousWordStart),
			
			"extend_next_word_start" => Ok(ExtendNextWordStart),
			"extend_next_word_end" => Ok(ExtendNextWordEnd),
			"extend_previous_word_start" => Ok(ExtendPreviousWordStart),
			
			"extend_line_below" => Ok(ExtendLineBelow),
			"extend_line_above" => Ok(ExtendLineAbove),
			
			_ => Err(()),
		}
	}
}

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
		self.cursors = new_cursors.collect();
		
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
	
	let utf8 = str::from_utf8(selection).ok()
		.filter(|_| selection.len() == 1)
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

// helpers
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
