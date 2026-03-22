use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::{action::{Action, AppAction, BufferAction, CursorAction}, buffer::{Mode, PartialAction}};

pub struct Config(
	pub HashMap<Mode, ModeConfig>
);

pub struct ModeConfig(
	pub HashMap<Option<PartialAction>, Keybinds>
);

pub struct Keybinds(
	pub HashMap<Keypress, Action>
);

#[derive(PartialEq, Eq, Hash)]
pub struct Keypress {
	code: KeyCode,
	modifiers: KeyModifiers
}

impl<const N: usize> From<[(Mode, ModeConfig); N]> for Config {
	fn from(array: [(Mode, ModeConfig); N]) -> Self {
		Self(array.into())
	}
}

impl<const N: usize> From<[(Option<PartialAction>, Keybinds); N]> for ModeConfig {
	fn from(array: [(Option<PartialAction>, Keybinds); N]) -> Self {
		Self(array.into())
	}
}

impl<const N: usize> From<[(Keypress, Action); N]> for Keybinds {
	fn from(array: [(Keypress, Action); N]) -> Self {
		Self(array.into())
	}
}

impl From<KeyCode> for Keypress {
	fn from(key_code: KeyCode) -> Self {
		Self {
			code: key_code,
			modifiers: KeyModifiers::NONE
		}
	}
}

const fn modifier_from_character(character: char) -> Option<KeyModifiers> {
	match character {
		'A' => Some(KeyModifiers::ALT),
		'C' => Some(KeyModifiers::CONTROL),
		_ => None
	}
}

impl TryFrom<&str> for Keypress {
	type Error = ();
	
	fn try_from(string: &str) -> Result<Self, ()> {
		match string.len() {
			3 => {
				Ok(Self {
					code: KeyCode::Char(
						string.chars().nth(2).unwrap()
					),
					modifiers: modifier_from_character(
						string.chars().nth(0).unwrap()
					).ok_or(())?,
				})
			}
			1 => {
				Ok(
					KeyCode::Char(
						string.chars().nth(0).unwrap()
					).into()
				)
			}
			_ => Err(())
		}
	}
}

impl From<KeyEvent> for Keypress {
	fn from(event: KeyEvent) -> Self {
		Self {
			code: event.code,
			modifiers: event.modifiers.difference(KeyModifiers::SHIFT),
		}
	}
}

impl Default for Config {
	#[allow(clippy::too_many_lines)]
	fn default() -> Self {
		[
			(Mode::Normal, [
				(None, [
					("q".try_into().unwrap(), AppAction::QuitIfSaved.into()),
					("Q".try_into().unwrap(), AppAction::Quit.into()),
					
					("v".try_into().unwrap(), BufferAction::SelectMode.into()),
					
					("g".try_into().unwrap(), BufferAction::Goto.into()),
					("z".try_into().unwrap(), BufferAction::View.into()),
					("r".try_into().unwrap(), BufferAction::Replace.into()),
					(" ".try_into().unwrap(), BufferAction::Space.into()),
					("*".try_into().unwrap(), BufferAction::Repeat.into()),
					("t".try_into().unwrap(), BufferAction::To.into()),
					
					("i".try_into().unwrap(), CursorAction::MoveByteUp.into()),
					("k".try_into().unwrap(), CursorAction::MoveByteDown.into()),
					("j".try_into().unwrap(), CursorAction::MoveByteLeft.into()),
					("l".try_into().unwrap(), CursorAction::MoveByteRight.into()),
					
					("G".try_into().unwrap(), CursorAction::GotoFileEnd.into()),
					
					("C-e".try_into().unwrap(), BufferAction::ScrollDown.into()),
					("C-y".try_into().unwrap(), BufferAction::ScrollUp.into()),
					
					("C-d".try_into().unwrap(), BufferAction::PageCursorHalfDown.into()),
					("C-u".try_into().unwrap(), BufferAction::PageCursorHalfUp.into()),
					
					("C-f".try_into().unwrap(), BufferAction::PageDown.into()),
					("C-b".try_into().unwrap(), BufferAction::PageUp.into()),
					
					("w".try_into().unwrap(), CursorAction::MoveNextWordStart.into()),
					("e".try_into().unwrap(), CursorAction::MoveNextWordEnd.into()),
					("b".try_into().unwrap(), CursorAction::MovePreviousWordStart.into()),
					
					(";".try_into().unwrap(), BufferAction::CollapseSelection.into()),
					("A-;".try_into().unwrap(), BufferAction::FlipSelections.into()),
					
					("x".try_into().unwrap(), CursorAction::ExtendLineBelow.into()),
					("X".try_into().unwrap(), CursorAction::ExtendLineAbove.into()),
					
					("d".try_into().unwrap(), BufferAction::Delete.into()),
					
					("u".try_into().unwrap(), BufferAction::Undo.into()),
					("U".try_into().unwrap(), BufferAction::Redo.into()),
					
					("C-j".try_into().unwrap(), AppAction::PreviousBuffer.into()),
					("C-l".try_into().unwrap(), AppAction::NextBuffer.into()),
					
					("C".try_into().unwrap(), BufferAction::CopySelectionOnNextLine.into()),
					
					("(".try_into().unwrap(), BufferAction::RotateSelectionsBackward.into()),
					(")".try_into().unwrap(), BufferAction::RotateSelectionsForward.into()),
					
					(",".try_into().unwrap(), BufferAction::KeepPrimarySelection.into()),
					("A-,".try_into().unwrap(), BufferAction::RemovePrimarySelection.into()),
					
					("1".try_into().unwrap(), BufferAction::SplitSelectionsInto1s.into()),
					("2".try_into().unwrap(), BufferAction::SplitSelectionsInto2s.into()),
					("3".try_into().unwrap(), BufferAction::SplitSelectionsInto3s.into()),
					("4".try_into().unwrap(), BufferAction::SplitSelectionsInto4s.into()),
					("5".try_into().unwrap(), BufferAction::SplitSelectionsInto5s.into()),
					("6".try_into().unwrap(), BufferAction::SplitSelectionsInto6s.into()),
					("7".try_into().unwrap(), BufferAction::SplitSelectionsInto7s.into()),
					("8".try_into().unwrap(), BufferAction::SplitSelectionsInto8s.into()),
					("9".try_into().unwrap(), BufferAction::SplitSelectionsInto9s.into()),
					
					("J".try_into().unwrap(), BufferAction::JumpToSelectedOffsetRelativeToMark.into()),
					("A-J".try_into().unwrap(), BufferAction::JumpToSelectedOffset.into()),
					
					("m".try_into().unwrap(), BufferAction::ToggleMark.into()),
					
					("y".try_into().unwrap(), AppAction::Yank.into()),
					
					("C- ".try_into().unwrap(), BufferAction::InspectSelection.into()),
				].into()),
				(Some(PartialAction::Goto), [
					("j".try_into().unwrap(), CursorAction::GotoLineStart.into()),
					("l".try_into().unwrap(), CursorAction::GotoLineEnd.into()),
					
					("g".try_into().unwrap(), CursorAction::GotoFileStart.into()),
				].into()),
				(Some(PartialAction::View), [
					("z".try_into().unwrap(), BufferAction::AlignViewCenter.into()),
					("b".try_into().unwrap(), BufferAction::AlignViewBottom.into()),
					("t".try_into().unwrap(), BufferAction::AlignViewTop.into()),
				].into()),
				(Some(PartialAction::Space), [
					("w".try_into().unwrap(), BufferAction::Save.into()),
				].into()),
				(Some(PartialAction::Repeat), [
					("i".try_into().unwrap(), CursorAction::MoveByteUp.into()),
					("k".try_into().unwrap(), CursorAction::MoveByteDown.into()),
					("j".try_into().unwrap(), CursorAction::MoveByteLeft.into()),
					("l".try_into().unwrap(), CursorAction::MoveByteRight.into()),
					
					("C-e".try_into().unwrap(), BufferAction::ScrollDown.into()),
					("C-y".try_into().unwrap(), BufferAction::ScrollUp.into()),
					
					("C-d".try_into().unwrap(), BufferAction::PageCursorHalfDown.into()),
					("C-u".try_into().unwrap(), BufferAction::PageCursorHalfUp.into()),
					
					("C-f".try_into().unwrap(), BufferAction::PageDown.into()),
					("C-b".try_into().unwrap(), BufferAction::PageUp.into()),
					
					("w".try_into().unwrap(), CursorAction::MoveNextWordStart.into()),
					("e".try_into().unwrap(), CursorAction::MoveNextWordEnd.into()),
					("b".try_into().unwrap(), CursorAction::MovePreviousWordStart.into()),
					
					("x".try_into().unwrap(), CursorAction::ExtendLineBelow.into()),
					("X".try_into().unwrap(), CursorAction::ExtendLineAbove.into()),
					
					("d".try_into().unwrap(), BufferAction::Delete.into()),
					
					("C".try_into().unwrap(), BufferAction::CopySelectionOnNextLine.into()),
				].into()),
				(Some(PartialAction::To), [
					("m".try_into().unwrap(), BufferAction::ExtendToMark.into()),
					("0".try_into().unwrap(), BufferAction::ExtendToNull.into()),
					("f".try_into().unwrap(), BufferAction::ExtendToFF.into()),
				].into()),
			].into()),
			(Mode::Select, [
				(None, [
					("q".try_into().unwrap(), AppAction::QuitIfSaved.into()),
					("Q".try_into().unwrap(), AppAction::Quit.into()),
					
					("v".try_into().unwrap(), BufferAction::NormalMode.into()),
					
					("g".try_into().unwrap(), BufferAction::Goto.into()),
					("z".try_into().unwrap(), BufferAction::View.into()),
					("r".try_into().unwrap(), BufferAction::Replace.into()),
					(" ".try_into().unwrap(), BufferAction::Space.into()),
					("*".try_into().unwrap(), BufferAction::Repeat.into()),
					("t".try_into().unwrap(), BufferAction::To.into()),
					
					("i".try_into().unwrap(), CursorAction::ExtendByteUp.into()),
					("k".try_into().unwrap(), CursorAction::ExtendByteDown.into()),
					("j".try_into().unwrap(), CursorAction::ExtendByteLeft.into()),
					("l".try_into().unwrap(), CursorAction::ExtendByteRight.into()),
					
					("C-e".try_into().unwrap(), BufferAction::ScrollDown.into()),
					("C-y".try_into().unwrap(), BufferAction::ScrollUp.into()),
					
					("C-d".try_into().unwrap(), BufferAction::PageCursorHalfDown.into()),
					("C-u".try_into().unwrap(), BufferAction::PageCursorHalfUp.into()),
					
					("C-f".try_into().unwrap(), BufferAction::PageDown.into()),
					("C-b".try_into().unwrap(), BufferAction::PageUp.into()),
					
					("w".try_into().unwrap(), CursorAction::ExtendNextWordStart.into()),
					("e".try_into().unwrap(), CursorAction::ExtendNextWordEnd.into()),
					("b".try_into().unwrap(), CursorAction::ExtendPreviousWordStart.into()),
					
					(";".try_into().unwrap(), BufferAction::CollapseSelection.into()),
					("A-;".try_into().unwrap(), BufferAction::FlipSelections.into()),
					
					("x".try_into().unwrap(), CursorAction::ExtendLineBelow.into()),
					("X".try_into().unwrap(), CursorAction::ExtendLineAbove.into()),
					
					("d".try_into().unwrap(), BufferAction::Delete.into()),
					
					("u".try_into().unwrap(), BufferAction::Undo.into()),
					("U".try_into().unwrap(), BufferAction::Redo.into()),
					
					("C".try_into().unwrap(), BufferAction::CopySelectionOnNextLine.into()),
					
					("(".try_into().unwrap(), BufferAction::RotateSelectionsBackward.into()),
					(")".try_into().unwrap(), BufferAction::RotateSelectionsForward.into()),
					
					(",".try_into().unwrap(), BufferAction::KeepPrimarySelection.into()),
					("A-,".try_into().unwrap(), BufferAction::RemovePrimarySelection.into()),
					
					("1".try_into().unwrap(), BufferAction::SplitSelectionsInto1s.into()),
					("2".try_into().unwrap(), BufferAction::SplitSelectionsInto2s.into()),
					("3".try_into().unwrap(), BufferAction::SplitSelectionsInto3s.into()),
					("4".try_into().unwrap(), BufferAction::SplitSelectionsInto4s.into()),
					("5".try_into().unwrap(), BufferAction::SplitSelectionsInto5s.into()),
					("6".try_into().unwrap(), BufferAction::SplitSelectionsInto6s.into()),
					("7".try_into().unwrap(), BufferAction::SplitSelectionsInto7s.into()),
					("8".try_into().unwrap(), BufferAction::SplitSelectionsInto8s.into()),
					("9".try_into().unwrap(), BufferAction::SplitSelectionsInto9s.into()),
					
					("J".try_into().unwrap(), BufferAction::JumpToSelectedOffsetRelativeToMark.into()),
					("A-J".try_into().unwrap(), BufferAction::JumpToSelectedOffset.into()),
					
					("m".try_into().unwrap(), BufferAction::ToggleMark.into()),
					
					("y".try_into().unwrap(), AppAction::Yank.into()),
					
					("C- ".try_into().unwrap(), BufferAction::InspectSelection.into()),
				].into()),
				(Some(PartialAction::View), [
					("z".try_into().unwrap(), BufferAction::AlignViewCenter.into()),
					("b".try_into().unwrap(), BufferAction::AlignViewBottom.into()),
					("t".try_into().unwrap(), BufferAction::AlignViewTop.into()),
				].into()),
				(Some(PartialAction::Space), [
					("w".try_into().unwrap(), BufferAction::Save.into()),
				].into()),
				(Some(PartialAction::Repeat), [
					("i".try_into().unwrap(), CursorAction::ExtendByteUp.into()),
					("k".try_into().unwrap(), CursorAction::ExtendByteDown.into()),
					("j".try_into().unwrap(), CursorAction::ExtendByteLeft.into()),
					("l".try_into().unwrap(), CursorAction::ExtendByteRight.into()),
					
					("C-e".try_into().unwrap(), BufferAction::ScrollDown.into()),
					("C-y".try_into().unwrap(), BufferAction::ScrollUp.into()),
					
					("C-d".try_into().unwrap(), BufferAction::PageCursorHalfDown.into()),
					("C-u".try_into().unwrap(), BufferAction::PageCursorHalfUp.into()),
					
					("C-f".try_into().unwrap(), BufferAction::PageDown.into()),
					("C-b".try_into().unwrap(), BufferAction::PageUp.into()),
					
					("w".try_into().unwrap(), CursorAction::ExtendNextWordStart.into()),
					("e".try_into().unwrap(), CursorAction::ExtendNextWordEnd.into()),
					("b".try_into().unwrap(), CursorAction::ExtendPreviousWordStart.into()),
					
					("x".try_into().unwrap(), CursorAction::ExtendLineBelow.into()),
					("X".try_into().unwrap(), CursorAction::ExtendLineAbove.into()),
					
					("d".try_into().unwrap(), BufferAction::Delete.into()),
					
					("C".try_into().unwrap(), BufferAction::CopySelectionOnNextLine.into()),
				].into()),
				(Some(PartialAction::To), [
					("m".try_into().unwrap(), BufferAction::ExtendToMark.into()),
					("0".try_into().unwrap(), BufferAction::ExtendToNull.into()),
					("f".try_into().unwrap(), BufferAction::ExtendToFF.into()),
				].into()),
			].into())
		].into()
	}
}
