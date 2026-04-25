use serde::{Deserialize, Serialize};

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
	
	FindTillMark,
	FindTillNull,
	FindTillFF,
	
	ExtendTillMark,
	ExtendTillNull,
	ExtendTillFF,
	
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
			
			FindTillMark => true,
			FindTillNull => true,
			FindTillFF => true,
			
			ExtendTillMark => true,
			ExtendTillNull => true,
			ExtendTillFF => true,
			
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
			
			FindTillMark => "find_till_mark",
			FindTillNull => "find_till_null",
			FindTillFF => "find_till_ff",
			
			ExtendTillMark => "extend_till_mark",
			ExtendTillNull => "extend_till_null",
			ExtendTillFF => "extend_till_ff",
			
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
			
			"find_till_mark" => Ok(FindTillMark),
			"find_till_null" => Ok(FindTillNull),
			"find_till_ff" => Ok(FindTillFF),
			
			"extend_till_mark" => Ok(ExtendTillMark),
			"extend_till_null" => Ok(ExtendTillNull),
			"extend_till_ff" => Ok(ExtendTillFF),
			
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
	
	ExtendLineStart,
	ExtendLineEnd,
	ExtendFileStart,
	ExtendFileEnd,
	
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
			
			ExtendLineStart => true,
			ExtendLineEnd => true,
			ExtendFileStart => true,
			ExtendFileEnd => true,
			
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
			
			ExtendLineStart => "extend_line_start",
			ExtendLineEnd => "extend_line_end",
			ExtendFileStart => "extend_file_start",
			ExtendFileEnd => "extend_file_end",
			
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
			
			"extend_line_start" => Ok(ExtendLineStart),
			"extend_line_end" => Ok(ExtendLineEnd),
			"extend_file_start" => Ok(ExtendFileStart),
			"extend_file_end" => Ok(ExtendFileEnd),
			
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
