use crate::{action::{AppAction, BufferAction, CursorAction}, buffer::{Mode, PartialAction}, config::{Config, Keypress}};

fn keypress(string: &str) -> Keypress {
	string.try_into().unwrap()
}

impl Default for Config {
	#[allow(clippy::too_many_lines)]
	fn default() -> Self {
		use AppAction::*;
		use BufferAction::*;
		use CursorAction::*;
		
		[
			(Mode::Normal, [
				(None, [
					(keypress("q"), QuitIfSaved.into()),
					(keypress("Q"), Quit.into()),
					
					(keypress("v"), SelectMode.into()),
					
					(keypress("g"), Goto.into()),
					(keypress("z"), View.into()),
					(keypress("r"), Replace.into()),
					(keypress(" "), Space.into()),
					(keypress("*"), Repeat.into()),
					(keypress("t"), To.into()),
					
					(keypress("i"), MoveByteUp.into()),
					(keypress("k"), MoveByteDown.into()),
					(keypress("j"), MoveByteLeft.into()),
					(keypress("l"), MoveByteRight.into()),
					
					(keypress("up"), MoveByteUp.into()),
					(keypress("down"), MoveByteDown.into()),
					(keypress("left"), MoveByteLeft.into()),
					(keypress("right"), MoveByteRight.into()),
					
					(keypress("G"), GotoFileEnd.into()),
					
					(keypress("C-e"), ScrollDown.into()),
					(keypress("C-y"), ScrollUp.into()),
					
					(keypress("C-d"), PageCursorHalfDown.into()),
					(keypress("C-u"), PageCursorHalfUp.into()),
					
					(keypress("C-f"), PageDown.into()),
					(keypress("C-b"), PageUp.into()),
					
					(keypress("w"), MoveNextWordStart.into()),
					(keypress("e"), MoveNextWordEnd.into()),
					(keypress("b"), MovePreviousWordStart.into()),
					
					(keypress(";"), CollapseSelection.into()),
					(keypress("A-;"), FlipSelections.into()),
					
					(keypress("x"), ExtendLineBelow.into()),
					(keypress("X"), ExtendLineAbove.into()),
					
					(keypress("d"), Delete.into()),
					
					(keypress("u"), Undo.into()),
					(keypress("U"), Redo.into()),
					
					(keypress("C-j"), PreviousBuffer.into()),
					(keypress("C-l"), NextBuffer.into()),
					
					(keypress("C"), CopySelectionOnNextLine.into()),
					
					(keypress("("), RotateSelectionsBackward.into()),
					(keypress(")"), RotateSelectionsForward.into()),
					
					(keypress(","), KeepPrimarySelection.into()),
					(keypress("A-,"), RemovePrimarySelection.into()),
					
					(keypress("1"), SplitSelectionsInto1s.into()),
					(keypress("2"), SplitSelectionsInto2s.into()),
					(keypress("3"), SplitSelectionsInto3s.into()),
					(keypress("4"), SplitSelectionsInto4s.into()),
					(keypress("5"), SplitSelectionsInto5s.into()),
					(keypress("6"), SplitSelectionsInto6s.into()),
					(keypress("7"), SplitSelectionsInto7s.into()),
					(keypress("8"), SplitSelectionsInto8s.into()),
					(keypress("9"), SplitSelectionsInto9s.into()),
					
					(keypress("J"), JumpToSelectedOffsetRelativeToMark.into()),
					(keypress("A-J"), JumpToSelectedOffset.into()),
					
					(keypress("m"), ToggleMark.into()),
					
					(keypress("y"), Yank.into()),
					
					(keypress("C- "), InspectSelection.into()),
					(keypress("A- "), InspectSelectionColor.into()),
				].into()),
				(Some(PartialAction::Goto), [
					(keypress("j"), GotoLineStart.into()),
					(keypress("l"), GotoLineEnd.into()),
					
					(keypress("g"), GotoFileStart.into()),
				].into()),
				(Some(PartialAction::View), [
					(keypress("z"), AlignViewCenter.into()),
					(keypress("b"), AlignViewBottom.into()),
					(keypress("t"), AlignViewTop.into()),
				].into()),
				(Some(PartialAction::Space), [
					(keypress("w"), Save.into()),
				].into()),
				(Some(PartialAction::Repeat), [
					(keypress("i"), MoveByteUp.into()),
					(keypress("k"), MoveByteDown.into()),
					(keypress("j"), MoveByteLeft.into()),
					(keypress("l"), MoveByteRight.into()),
					
					(keypress("up"), MoveByteUp.into()),
					(keypress("down"), MoveByteDown.into()),
					(keypress("left"), MoveByteLeft.into()),
					(keypress("right"), MoveByteRight.into()),
					
					(keypress("C-e"), ScrollDown.into()),
					(keypress("C-y"), ScrollUp.into()),
					
					(keypress("C-d"), PageCursorHalfDown.into()),
					(keypress("C-u"), PageCursorHalfUp.into()),
					
					(keypress("C-f"), PageDown.into()),
					(keypress("C-b"), PageUp.into()),
					
					(keypress("w"), MoveNextWordStart.into()),
					(keypress("e"), MoveNextWordEnd.into()),
					(keypress("b"), MovePreviousWordStart.into()),
					
					(keypress("x"), ExtendLineBelow.into()),
					(keypress("X"), ExtendLineAbove.into()),
					
					(keypress("d"), Delete.into()),
					
					(keypress("C"), CopySelectionOnNextLine.into()),
				].into()),
				(Some(PartialAction::Till), [
					(keypress("m"), FindTillMark.into()),
					(keypress("0"), FindTillNull.into()),
					(keypress("f"), FindTillFF.into()),
				].into()),
			].into()),
			(Mode::Select, [
				(None, [
					(keypress("q"), QuitIfSaved.into()),
					(keypress("Q"), Quit.into()),
					
					(keypress("v"), NormalMode.into()),
					
					(keypress("g"), Goto.into()),
					(keypress("z"), View.into()),
					(keypress("r"), Replace.into()),
					(keypress(" "), Space.into()),
					(keypress("*"), Repeat.into()),
					(keypress("t"), To.into()),
					
					(keypress("i"), ExtendByteUp.into()),
					(keypress("k"), ExtendByteDown.into()),
					(keypress("j"), ExtendByteLeft.into()),
					(keypress("l"), ExtendByteRight.into()),
					
					(keypress("G"), ExtendFileEnd.into()),
					
					(keypress("up"), ExtendByteUp.into()),
					(keypress("down"), ExtendByteDown.into()),
					(keypress("left"), ExtendByteLeft.into()),
					(keypress("right"), ExtendByteRight.into()),
					
					(keypress("C-e"), ScrollDown.into()),
					(keypress("C-y"), ScrollUp.into()),
					
					(keypress("C-d"), PageCursorHalfDown.into()),
					(keypress("C-u"), PageCursorHalfUp.into()),
					
					(keypress("C-f"), PageDown.into()),
					(keypress("C-b"), PageUp.into()),
					
					(keypress("w"), ExtendNextWordStart.into()),
					(keypress("e"), ExtendNextWordEnd.into()),
					(keypress("b"), ExtendPreviousWordStart.into()),
					
					(keypress(";"), CollapseSelection.into()),
					(keypress("A-;"), FlipSelections.into()),
					
					(keypress("x"), ExtendLineBelow.into()),
					(keypress("X"), ExtendLineAbove.into()),
					
					(keypress("d"), Delete.into()),
					
					(keypress("u"), Undo.into()),
					(keypress("U"), Redo.into()),
					
					(keypress("C"), CopySelectionOnNextLine.into()),
					
					(keypress("("), RotateSelectionsBackward.into()),
					(keypress(")"), RotateSelectionsForward.into()),
					
					(keypress(","), KeepPrimarySelection.into()),
					(keypress("A-,"), RemovePrimarySelection.into()),
					
					(keypress("1"), SplitSelectionsInto1s.into()),
					(keypress("2"), SplitSelectionsInto2s.into()),
					(keypress("3"), SplitSelectionsInto3s.into()),
					(keypress("4"), SplitSelectionsInto4s.into()),
					(keypress("5"), SplitSelectionsInto5s.into()),
					(keypress("6"), SplitSelectionsInto6s.into()),
					(keypress("7"), SplitSelectionsInto7s.into()),
					(keypress("8"), SplitSelectionsInto8s.into()),
					(keypress("9"), SplitSelectionsInto9s.into()),
					
					(keypress("J"), JumpToSelectedOffsetRelativeToMark.into()),
					(keypress("A-J"), JumpToSelectedOffset.into()),
					
					(keypress("m"), ToggleMark.into()),
					
					(keypress("y"), Yank.into()),
					
					(keypress("C- "), InspectSelection.into()),
					(keypress("A- "), InspectSelectionColor.into()),
				].into()),
				(Some(PartialAction::Goto), [
					(keypress("j"), ExtendLineStart.into()),
					(keypress("l"), ExtendLineEnd.into()),
					
					(keypress("g"), ExtendFileStart.into()),
				].into()),
				(Some(PartialAction::View), [
					(keypress("z"), AlignViewCenter.into()),
					(keypress("b"), AlignViewBottom.into()),
					(keypress("t"), AlignViewTop.into()),
				].into()),
				(Some(PartialAction::Space), [
					(keypress("w"), Save.into()),
				].into()),
				(Some(PartialAction::Repeat), [
					(keypress("i"), ExtendByteUp.into()),
					(keypress("k"), ExtendByteDown.into()),
					(keypress("j"), ExtendByteLeft.into()),
					(keypress("l"), ExtendByteRight.into()),
					
					(keypress("up"), ExtendByteUp.into()),
					(keypress("down"), ExtendByteDown.into()),
					(keypress("left"), ExtendByteLeft.into()),
					(keypress("right"), ExtendByteRight.into()),
					
					(keypress("C-e"), ScrollDown.into()),
					(keypress("C-y"), ScrollUp.into()),
					
					(keypress("C-d"), PageCursorHalfDown.into()),
					(keypress("C-u"), PageCursorHalfUp.into()),
					
					(keypress("C-f"), PageDown.into()),
					(keypress("C-b"), PageUp.into()),
					
					(keypress("w"), ExtendNextWordStart.into()),
					(keypress("e"), ExtendNextWordEnd.into()),
					(keypress("b"), ExtendPreviousWordStart.into()),
					
					(keypress("x"), ExtendLineBelow.into()),
					(keypress("X"), ExtendLineAbove.into()),
					
					(keypress("d"), Delete.into()),
					
					(keypress("C"), CopySelectionOnNextLine.into()),
				].into()),
				(Some(PartialAction::Till), [
					(keypress("m"), ExtendTillMark.into()),
					(keypress("0"), ExtendTillNull.into()),
					(keypress("f"), ExtendTillFF.into()),
				].into()),
			].into())
		].into()
	}
}
