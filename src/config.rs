use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::{action::Action, buffer::{Mode, PartialAction}};

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
			modifiers: match event.modifiers {
				KeyModifiers::SHIFT => KeyModifiers::NONE,
				x => x,
			},
		}
	}
}

impl Default for Config {
	fn default() -> Self {
		[
			(Mode::Normal, [
				(None, [
					("q".try_into().unwrap(), Action::QuitIfSaved),
					("Q".try_into().unwrap(), Action::Quit),
					
					("v".try_into().unwrap(), Action::SelectMode),
					
					("g".try_into().unwrap(), Action::Goto),
					("z".try_into().unwrap(), Action::View),
					("r".try_into().unwrap(), Action::Replace),
					(" ".try_into().unwrap(), Action::Space),
					
					("i".try_into().unwrap(), Action::MoveByteUp),
					("k".try_into().unwrap(), Action::MoveByteDown),
					("j".try_into().unwrap(), Action::MoveByteLeft),
					("l".try_into().unwrap(), Action::MoveByteRight),
					
					("G".try_into().unwrap(), Action::GotoFileEnd),
					
					("C-e".try_into().unwrap(), Action::ScrollDown),
					("C-y".try_into().unwrap(), Action::ScrollUp),
					
					("C-d".try_into().unwrap(), Action::PageCursorHalfDown),
					("C-u".try_into().unwrap(), Action::PageCursorHalfUp),
					
					("C-f".try_into().unwrap(), Action::PageDown),
					("C-b".try_into().unwrap(), Action::PageUp),
					
					("w".try_into().unwrap(), Action::MoveNextWordStart),
					("e".try_into().unwrap(), Action::MoveNextWordEnd),
					("b".try_into().unwrap(), Action::MovePreviousWordStart),
					
					(";".try_into().unwrap(), Action::CollapseSelection),
					
					("x".try_into().unwrap(), Action::ExtendLineBelow),
					("X".try_into().unwrap(), Action::ExtendLineAbove),
					
					("d".try_into().unwrap(), Action::Delete),
					
					("u".try_into().unwrap(), Action::Undo),
					("U".try_into().unwrap(), Action::Redo),
				].into()),
				(Some(PartialAction::Goto), [
					("j".try_into().unwrap(), Action::GotoLineStart),
					("l".try_into().unwrap(), Action::GotoLineEnd),
					
					("g".try_into().unwrap(), Action::GotoFileStart),
				].into()),
				(Some(PartialAction::Space), [
					("w".try_into().unwrap(), Action::Save),
				].into()),
			].into()),
			(Mode::Select, [
				(None, [
					("q".try_into().unwrap(), Action::QuitIfSaved),
					("Q".try_into().unwrap(), Action::Quit),
					
					("v".try_into().unwrap(), Action::NormalMode),
					
					("g".try_into().unwrap(), Action::Goto),
					("z".try_into().unwrap(), Action::View),
					("r".try_into().unwrap(), Action::Replace),
					(" ".try_into().unwrap(), Action::Space),
					
					("i".try_into().unwrap(), Action::ExtendByteUp),
					("k".try_into().unwrap(), Action::ExtendByteDown),
					("j".try_into().unwrap(), Action::ExtendByteLeft),
					("l".try_into().unwrap(), Action::ExtendByteRight),
					
					("C-e".try_into().unwrap(), Action::ScrollDown),
					("C-y".try_into().unwrap(), Action::ScrollUp),
					
					("C-d".try_into().unwrap(), Action::PageCursorHalfDown),
					("C-u".try_into().unwrap(), Action::PageCursorHalfUp),
					
					("C-f".try_into().unwrap(), Action::PageDown),
					("C-b".try_into().unwrap(), Action::PageUp),
					
					("w".try_into().unwrap(), Action::ExtendNextWordStart),
					("e".try_into().unwrap(), Action::ExtendNextWordEnd),
					("b".try_into().unwrap(), Action::ExtendPreviousWordStart),
					
					(";".try_into().unwrap(), Action::CollapseSelection),
					
					("x".try_into().unwrap(), Action::ExtendLineBelow),
					("X".try_into().unwrap(), Action::ExtendLineAbove),
					
					("d".try_into().unwrap(), Action::Delete),
					
					("u".try_into().unwrap(), Action::Undo),
					("U".try_into().unwrap(), Action::Redo),
				].into()),
				(Some(PartialAction::Space), [
					("w".try_into().unwrap(), Action::Save),
				].into()),
			].into())
		].into()
	}
}
