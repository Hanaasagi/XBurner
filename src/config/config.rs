use std::collections::HashMap;
use std::fmt;
use std::fs;

use evdev::Key;
use indexmap::IndexMap;
use serde;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};

use super::parser::parse_key_combo;
use super::parser::parse_modmap;
use super::parser::string_or_vec;

#[derive(Debug, Clone)]
pub enum Modifier {
    Shift,
    Control,
    Alt,
    Windows,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyCombo {
    pub key: Key,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub windows: bool,
}

impl<'de> Deserialize<'de> for KeyCombo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyPressVisitor;

        impl<'de> Visitor<'de> for KeyPressVisitor {
            type Value = KeyCombo;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                parse_key_combo(value).map_err(Error::custom)
            }
        }

        deserializer.deserialize_any(KeyPressVisitor)
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Remap(KeyCombo),
    Shell(String),
}

#[derive(Clone, Debug, Deserialize)]
pub struct KeyBinding {
    #[serde(rename = "key")]
    pub key_combo: KeyCombo,
    pub shell: Option<String>,
    pub remap: Option<KeyCombo>,
    pub desc: Option<String>,
}

impl KeyBinding {
    #[inline]
    pub fn get_action(&self) -> Action {
        if self.shell.is_some() {
            return Action::Shell(self.shell.as_ref().unwrap().to_string());
        }
        if self.remap.is_some() {
            return Action::Remap(self.remap.clone().unwrap());
        }
        unreachable!();
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Group {
    pub key_bindings: Vec<KeyBinding>,
    #[serde(default, deserialize_with = "string_or_vec", rename = "in")]
    pub in_: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_vec", rename = "notin")]
    pub not_in: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mode {
    pub groups: Vec<String>,
    pub switch_key: Option<KeyCombo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Options {
    pub mode_switch_key: Option<KeyCombo>,
    pub default_mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default, deserialize_with = "parse_modmap")]
    pub modmap: Option<HashMap<Key, Key>>,
    pub modes: Option<IndexMap<String, Mode>>,
    pub groups: IndexMap<String, Group>,
    pub options: Option<Options>,
}

impl Config {
    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let yaml = fs::read_to_string(&filename)?;
        let config: Config = serde_yaml::from_str(&yaml)?;
        return Ok(config);
    }
}
