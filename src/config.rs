use std::{collections::{HashMap, hash_map::Entry}, fmt::{self, Formatter}};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::{action::{Action, AppAction, BufferAction, CursorAction}, buffer::{Mode, PartialAction}};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::{Error, MapAccess, Unexpected, Visitor}, ser::SerializeMap};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Config(
	pub HashMap<Mode, ModeConfig>
);

#[derive(Debug)]
pub struct ModeConfig(
	pub HashMap<Option<PartialAction>, Keybinds>
);

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[serde(transparent)]
pub struct Keybinds(
	pub HashMap<Keypress, Action>
);

#[derive(PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[derive(Debug)]
#[serde(into = "String")]
#[serde(try_from = "&str")]
pub struct Keypress {
	code: KeyCode,
	modifiers: KeyModifiers
}

impl Serialize for ModeConfig {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut map = serializer.serialize_map(None)?;
		
		if let Some(keybinds) = self.0.get(&None) {
			for (keypress, action) in &keybinds.0 {
				map.serialize_entry(
					&String::from(keypress),
					&action
				)?;
			}
		}
		
		for (partial_action, keybinds) in &self.0 {
			let Some(partial_action) = partial_action else { continue };
			
			map.serialize_entry(
				partial_action.into(),
				keybinds
			)?;
		}
		
		map.end()
	}
}

struct ModeConfigVisitor;

impl<'de> Visitor<'de> for ModeConfigVisitor {
	type Value = ModeConfig;
	
	fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
		formatter.write_str("a config for keypresses in various partial action states")
	}
	
	fn visit_map<Map: MapAccess<'de>>(
		self,
		mut map: Map
	) -> Result<Self::Value, Map::Error> {
		let mut result = ModeConfig(HashMap::new());
		
		while let Some(key) = map.next_key::<&str>()? {
			if let Ok(partial_action) = PartialAction::try_from(key) {
				let keybinds: Keybinds = map.next_value()?;
				
				match result.0.entry(Some(partial_action)) {
					Entry::Occupied(mut occupied_entry) => {
						occupied_entry.get_mut().0.extend(keybinds.0);
					},
					Entry::Vacant(vacant_entry) => {
						vacant_entry.insert(keybinds);
					}
				}
			} else {
				let Ok(keypress) = key.try_into() else {
					return Err(Error::invalid_value(
						Unexpected::Str(key),
						&"a valid keypress, with an optional modifier"
					));
				};
				
				result.0.entry(None)
					.or_insert_with(|| Keybinds(HashMap::new()))
					.0.insert(keypress, map.next_value()?);
			}
		}
		
		Ok(result)
	}
}

impl<'de> Deserialize<'de> for ModeConfig {
	fn deserialize<D: Deserializer<'de>>(
		deserializer: D
	) -> Result<Self, D::Error> {
		deserializer.deserialize_map(ModeConfigVisitor)
	}
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

const fn str_from_modifiers(modifier: KeyModifiers) -> &'static str {
	match modifier {
		KeyModifiers::ALT => "A-",
		KeyModifiers::CONTROL => "C-",
		_ => ""
	}
}

fn string_from_code(code: KeyCode) -> String {
	match code {
		KeyCode::Char(character) => character.to_string(),
		KeyCode::Up => "Up".to_string(),
		KeyCode::Down => "Down".to_string(),
		KeyCode::Left => "Left".to_string(),
		KeyCode::Right => "Right".to_string(),
		_ => todo!()
	}
}

impl TryFrom<&str> for Keypress {
	type Error = String;
	
	fn try_from(string: &str) -> Result<Self, Self::Error> {
		match string.chars().count() {
			3 => {
				Ok(Self {
					code: KeyCode::Char(
						string.chars().nth(2).unwrap()
					),
					modifiers: modifier_from_character(
						string.chars().nth(0).unwrap()
					).ok_or(format!("unknown modifier: {}", string.chars().nth(0).unwrap()))?,
				})
			}
			1 => {
				Ok(
					KeyCode::Char(
						string.chars().nth(0).unwrap()
					).into()
				)
			}
			_ => Err(format!("invalid keypress: {}. only one modifier is allowed", string))
		}
	}
}

impl From<&Keypress> for String {
	fn from(value: &Keypress) -> Self {
		format!(
			"{}{}",
			str_from_modifiers(value.modifiers),
			string_from_code(value.code)
		)
	}
}

impl From<Keypress> for String {
	fn from(value: Keypress) -> Self {
		String::from(&value)
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
		use AppAction::*;
		use BufferAction::*;
		use CursorAction::*;
		
		[
			(Mode::Normal, [
				(None, [
					("q".try_into().unwrap(), QuitIfSaved.into()),
					("Q".try_into().unwrap(), Quit.into()),
					
					("v".try_into().unwrap(), SelectMode.into()),
					
					("g".try_into().unwrap(), Goto.into()),
					("z".try_into().unwrap(), View.into()),
					("r".try_into().unwrap(), Replace.into()),
					(" ".try_into().unwrap(), Space.into()),
					("*".try_into().unwrap(), Repeat.into()),
					("t".try_into().unwrap(), To.into()),
					
					("i".try_into().unwrap(), MoveByteUp.into()),
					("k".try_into().unwrap(), MoveByteDown.into()),
					("j".try_into().unwrap(), MoveByteLeft.into()),
					("l".try_into().unwrap(), MoveByteRight.into()),
					
					("G".try_into().unwrap(), GotoFileEnd.into()),
					
					("C-e".try_into().unwrap(), ScrollDown.into()),
					("C-y".try_into().unwrap(), ScrollUp.into()),
					
					("C-d".try_into().unwrap(), PageCursorHalfDown.into()),
					("C-u".try_into().unwrap(), PageCursorHalfUp.into()),
					
					("C-f".try_into().unwrap(), PageDown.into()),
					("C-b".try_into().unwrap(), PageUp.into()),
					
					("w".try_into().unwrap(), MoveNextWordStart.into()),
					("e".try_into().unwrap(), MoveNextWordEnd.into()),
					("b".try_into().unwrap(), MovePreviousWordStart.into()),
					
					(";".try_into().unwrap(), CollapseSelection.into()),
					("A-;".try_into().unwrap(), FlipSelections.into()),
					
					("x".try_into().unwrap(), ExtendLineBelow.into()),
					("X".try_into().unwrap(), ExtendLineAbove.into()),
					
					("d".try_into().unwrap(), Delete.into()),
					
					("u".try_into().unwrap(), Undo.into()),
					("U".try_into().unwrap(), Redo.into()),
					
					("C-j".try_into().unwrap(), PreviousBuffer.into()),
					("C-l".try_into().unwrap(), NextBuffer.into()),
					
					("C".try_into().unwrap(), CopySelectionOnNextLine.into()),
					
					("(".try_into().unwrap(), RotateSelectionsBackward.into()),
					(")".try_into().unwrap(), RotateSelectionsForward.into()),
					
					(",".try_into().unwrap(), KeepPrimarySelection.into()),
					("A-,".try_into().unwrap(), RemovePrimarySelection.into()),
					
					("1".try_into().unwrap(), SplitSelectionsInto1s.into()),
					("2".try_into().unwrap(), SplitSelectionsInto2s.into()),
					("3".try_into().unwrap(), SplitSelectionsInto3s.into()),
					("4".try_into().unwrap(), SplitSelectionsInto4s.into()),
					("5".try_into().unwrap(), SplitSelectionsInto5s.into()),
					("6".try_into().unwrap(), SplitSelectionsInto6s.into()),
					("7".try_into().unwrap(), SplitSelectionsInto7s.into()),
					("8".try_into().unwrap(), SplitSelectionsInto8s.into()),
					("9".try_into().unwrap(), SplitSelectionsInto9s.into()),
					
					("J".try_into().unwrap(), JumpToSelectedOffsetRelativeToMark.into()),
					("A-J".try_into().unwrap(), JumpToSelectedOffset.into()),
					
					("m".try_into().unwrap(), ToggleMark.into()),
					
					("y".try_into().unwrap(), Yank.into()),
					
					("C- ".try_into().unwrap(), InspectSelection.into()),
					("A- ".try_into().unwrap(), InspectSelectionColor.into()),
				].into()),
				(Some(PartialAction::Goto), [
					("j".try_into().unwrap(), GotoLineStart.into()),
					("l".try_into().unwrap(), GotoLineEnd.into()),
					
					("g".try_into().unwrap(), GotoFileStart.into()),
				].into()),
				(Some(PartialAction::View), [
					("z".try_into().unwrap(), AlignViewCenter.into()),
					("b".try_into().unwrap(), AlignViewBottom.into()),
					("t".try_into().unwrap(), AlignViewTop.into()),
				].into()),
				(Some(PartialAction::Space), [
					("w".try_into().unwrap(), Save.into()),
				].into()),
				(Some(PartialAction::Repeat), [
					("i".try_into().unwrap(), MoveByteUp.into()),
					("k".try_into().unwrap(), MoveByteDown.into()),
					("j".try_into().unwrap(), MoveByteLeft.into()),
					("l".try_into().unwrap(), MoveByteRight.into()),
					
					("C-e".try_into().unwrap(), ScrollDown.into()),
					("C-y".try_into().unwrap(), ScrollUp.into()),
					
					("C-d".try_into().unwrap(), PageCursorHalfDown.into()),
					("C-u".try_into().unwrap(), PageCursorHalfUp.into()),
					
					("C-f".try_into().unwrap(), PageDown.into()),
					("C-b".try_into().unwrap(), PageUp.into()),
					
					("w".try_into().unwrap(), MoveNextWordStart.into()),
					("e".try_into().unwrap(), MoveNextWordEnd.into()),
					("b".try_into().unwrap(), MovePreviousWordStart.into()),
					
					("x".try_into().unwrap(), ExtendLineBelow.into()),
					("X".try_into().unwrap(), ExtendLineAbove.into()),
					
					("d".try_into().unwrap(), Delete.into()),
					
					("C".try_into().unwrap(), CopySelectionOnNextLine.into()),
				].into()),
				(Some(PartialAction::To), [
					("m".try_into().unwrap(), ExtendToMark.into()),
					("0".try_into().unwrap(), ExtendToNull.into()),
					("f".try_into().unwrap(), ExtendToFF.into()),
				].into()),
			].into()),
			(Mode::Select, [
				(None, [
					("q".try_into().unwrap(), QuitIfSaved.into()),
					("Q".try_into().unwrap(), Quit.into()),
					
					("v".try_into().unwrap(), NormalMode.into()),
					
					("g".try_into().unwrap(), Goto.into()),
					("z".try_into().unwrap(), View.into()),
					("r".try_into().unwrap(), Replace.into()),
					(" ".try_into().unwrap(), Space.into()),
					("*".try_into().unwrap(), Repeat.into()),
					("t".try_into().unwrap(), To.into()),
					
					("i".try_into().unwrap(), ExtendByteUp.into()),
					("k".try_into().unwrap(), ExtendByteDown.into()),
					("j".try_into().unwrap(), ExtendByteLeft.into()),
					("l".try_into().unwrap(), ExtendByteRight.into()),
					
					("C-e".try_into().unwrap(), ScrollDown.into()),
					("C-y".try_into().unwrap(), ScrollUp.into()),
					
					("C-d".try_into().unwrap(), PageCursorHalfDown.into()),
					("C-u".try_into().unwrap(), PageCursorHalfUp.into()),
					
					("C-f".try_into().unwrap(), PageDown.into()),
					("C-b".try_into().unwrap(), PageUp.into()),
					
					("w".try_into().unwrap(), ExtendNextWordStart.into()),
					("e".try_into().unwrap(), ExtendNextWordEnd.into()),
					("b".try_into().unwrap(), ExtendPreviousWordStart.into()),
					
					(";".try_into().unwrap(), CollapseSelection.into()),
					("A-;".try_into().unwrap(), FlipSelections.into()),
					
					("x".try_into().unwrap(), ExtendLineBelow.into()),
					("X".try_into().unwrap(), ExtendLineAbove.into()),
					
					("d".try_into().unwrap(), Delete.into()),
					
					("u".try_into().unwrap(), Undo.into()),
					("U".try_into().unwrap(), Redo.into()),
					
					("C".try_into().unwrap(), CopySelectionOnNextLine.into()),
					
					("(".try_into().unwrap(), RotateSelectionsBackward.into()),
					(")".try_into().unwrap(), RotateSelectionsForward.into()),
					
					(",".try_into().unwrap(), KeepPrimarySelection.into()),
					("A-,".try_into().unwrap(), RemovePrimarySelection.into()),
					
					("1".try_into().unwrap(), SplitSelectionsInto1s.into()),
					("2".try_into().unwrap(), SplitSelectionsInto2s.into()),
					("3".try_into().unwrap(), SplitSelectionsInto3s.into()),
					("4".try_into().unwrap(), SplitSelectionsInto4s.into()),
					("5".try_into().unwrap(), SplitSelectionsInto5s.into()),
					("6".try_into().unwrap(), SplitSelectionsInto6s.into()),
					("7".try_into().unwrap(), SplitSelectionsInto7s.into()),
					("8".try_into().unwrap(), SplitSelectionsInto8s.into()),
					("9".try_into().unwrap(), SplitSelectionsInto9s.into()),
					
					("J".try_into().unwrap(), JumpToSelectedOffsetRelativeToMark.into()),
					("A-J".try_into().unwrap(), JumpToSelectedOffset.into()),
					
					("m".try_into().unwrap(), ToggleMark.into()),
					
					("y".try_into().unwrap(), Yank.into()),
					
					("C- ".try_into().unwrap(), InspectSelection.into()),
					("A- ".try_into().unwrap(), InspectSelectionColor.into()),
				].into()),
				(Some(PartialAction::View), [
					("z".try_into().unwrap(), AlignViewCenter.into()),
					("b".try_into().unwrap(), AlignViewBottom.into()),
					("t".try_into().unwrap(), AlignViewTop.into()),
				].into()),
				(Some(PartialAction::Space), [
					("w".try_into().unwrap(), Save.into()),
				].into()),
				(Some(PartialAction::Repeat), [
					("i".try_into().unwrap(), ExtendByteUp.into()),
					("k".try_into().unwrap(), ExtendByteDown.into()),
					("j".try_into().unwrap(), ExtendByteLeft.into()),
					("l".try_into().unwrap(), ExtendByteRight.into()),
					
					("C-e".try_into().unwrap(), ScrollDown.into()),
					("C-y".try_into().unwrap(), ScrollUp.into()),
					
					("C-d".try_into().unwrap(), PageCursorHalfDown.into()),
					("C-u".try_into().unwrap(), PageCursorHalfUp.into()),
					
					("C-f".try_into().unwrap(), PageDown.into()),
					("C-b".try_into().unwrap(), PageUp.into()),
					
					("w".try_into().unwrap(), ExtendNextWordStart.into()),
					("e".try_into().unwrap(), ExtendNextWordEnd.into()),
					("b".try_into().unwrap(), ExtendPreviousWordStart.into()),
					
					("x".try_into().unwrap(), ExtendLineBelow.into()),
					("X".try_into().unwrap(), ExtendLineAbove.into()),
					
					("d".try_into().unwrap(), Delete.into()),
					
					("C".try_into().unwrap(), CopySelectionOnNextLine.into()),
				].into()),
				(Some(PartialAction::To), [
					("m".try_into().unwrap(), ExtendToMark.into()),
					("0".try_into().unwrap(), ExtendToNull.into()),
					("f".try_into().unwrap(), ExtendToFF.into()),
				].into()),
			].into())
		].into()
	}
}
