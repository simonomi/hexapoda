use std::{collections::{HashMap, hash_map::Entry}, env::{self, home_dir}, fmt::{self, Formatter}, fs::read_to_string, io, path::PathBuf};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::{action::Action, buffer::{Mode, PartialAction}};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::{Error, MapAccess, Unexpected, Visitor}, ser::SerializeMap};

mod default;

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

impl Config {
	#[cfg(unix)]
	fn default_path() -> Option<PathBuf> {
		env::var_os("XDG_CONFIG_HOME")
			.map(PathBuf::from)
			.take_if(|xdg_config_home| xdg_config_home.is_absolute())
			.or_else(|| home_dir().map(|home| home.join(".config")))
			.map(|config_path| config_path.join("hexapoda.toml"))
	}
	
	#[cfg(windows)]
	fn default_path() -> Option<PathBuf> {
		// this isn't technically the right way but it should be good enough
		home_dir().map(|home| home.join("AppData").join("Roaming").join("hexapoda.toml"))
	}
	
	pub fn path(override_path: Option<PathBuf>) -> Option<PathBuf> {
		override_path.or_else(Self::default_path)
	}
	
	pub fn init(override_path: Option<PathBuf>) -> Result<Self, ConfigInitError> {
		let path = Self::path(override_path)
			.ok_or(ConfigInitError::NoConfigPath)?;
		
		let raw_config = read_to_string(path)?;
		
		Ok(Self::default().combined_with(toml::from_str(&raw_config)?))
	}
	
	fn combined_with(mut self, other: Self) -> Self {
		for (mode, mode_config) in other.0 {
			match self.0.entry(mode) {
				Entry::Occupied(mut occupied_entry) => {
					occupied_entry.get_mut().combine_with(mode_config);
				}
				Entry::Vacant(vacant_entry) => {
					vacant_entry.insert(mode_config);
				}
			}
		}
		self
	}
}

pub enum ConfigInitError {
	NoConfigPath, IO(io::Error), Deserialization(toml::de::Error)
}

impl From<io::Error> for ConfigInitError {
	fn from(error: io::Error) -> Self {
		Self::IO(error)
	}
}

impl From<toml::de::Error> for ConfigInitError {
	fn from(error: toml::de::Error) -> Self {
		Self::Deserialization(error)
	}
}

impl ModeConfig {
	fn combine_with(&mut self, other: Self) {
		for (partial_action, keybinds) in other.0 {
			match self.0.entry(partial_action) {
				Entry::Occupied(mut occupied_entry) => {
					occupied_entry.get_mut().combine_with(keybinds);
				}
				Entry::Vacant(vacant_entry) => {
					vacant_entry.insert(keybinds);
				}
			}
		}
	}
}

impl Keybinds {
	fn combine_with(&mut self, other: Self) {
		self.0.extend(other.0);
	}
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
				partial_action,
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

fn modifier_from_character(character: char) -> Result<KeyModifiers, String> {
	match character {
		'C' => Ok(KeyModifiers::CONTROL),
		'A' => Ok(KeyModifiers::ALT),
		'S' => Ok(KeyModifiers::SHIFT),
		_ => Err(format!("unknown modifier: {character}"))
	}
}

const fn str_from_modifiers(modifier: KeyModifiers) -> &'static str {
	match modifier {
		KeyModifiers::CONTROL => "C-",
		KeyModifiers::ALT => "A-",
		KeyModifiers::SHIFT => "S-",
		_ => ""
	}
}

fn string_from_code(code: KeyCode) -> String {
	use KeyCode::*;
	
	match code {
		Char(character) => character.to_string(),
		Backspace => "backspace".to_string(),
		Enter => "enter".to_string(),
		Up => "up".to_string(),
		Down => "down".to_string(),
		Left => "left".to_string(),
		Right => "right".to_string(),
		Home => "home".to_string(),
		End => "end".to_string(),
		PageUp => "pageup".to_string(),
		PageDown => "pagedown".to_string(),
		Tab => "tab".to_string(),
		Delete => "delete".to_string(),
		Insert => "insert".to_string(),
		Esc => "escape".to_string(),
		_ => todo!()
	}
}

impl TryFrom<&str> for Keypress {
	type Error = String;
	
	fn try_from(string: &str) -> Result<Self, Self::Error> {
		if string.is_empty() {
			return Err("keypress must not be empty".to_string());
		}
		
		let mut chunks = string.split('-');
		
		Ok(Self {
			code: try_key_code_from(chunks.next_back().unwrap())?,
			modifiers: chunks
				.map(|raw_modifier| {
					if raw_modifier.len() == 1 {
						modifier_from_character(
							raw_modifier.chars().next().unwrap()
						)
					} else {
						Err(format!("invalid modifier: {raw_modifier}"))
					}
				})
				.try_fold(KeyModifiers::NONE, |partial_result, modifier| {
					Ok::<KeyModifiers, String>(partial_result.union(modifier?))
				})?
		})
	}
}

fn try_key_code_from(string: &str) -> Result<KeyCode, String> {
	match string {
		"backspace" => Ok(KeyCode::Backspace),
		"enter" => Ok(KeyCode::Enter),
		"up" => Ok(KeyCode::Up),
		"down" => Ok(KeyCode::Down),
		"left" => Ok(KeyCode::Left),
		"right" => Ok(KeyCode::Right),
		"home" => Ok(KeyCode::Home),
		"end" => Ok(KeyCode::End),
		"pageup" => Ok(KeyCode::PageUp),
		"pagedown" => Ok(KeyCode::PageDown),
		"tab" => Ok(KeyCode::Tab),
		"delete" => Ok(KeyCode::Delete),
		"insert" => Ok(KeyCode::Insert),
		"escape" => Ok(KeyCode::Esc),
		character if character.len() == 1 => Ok(KeyCode::Char(character.chars().next().unwrap())),
		_ => Err(format!("invalid key code: '{string}'")),
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
		Self::from(&value)
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
