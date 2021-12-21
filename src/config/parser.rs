use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use evdev::Key;
use serde::de::{value, Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::config::KeyCombo;
use super::config::Modifier;

// Some parse utils for serde-yaml

pub fn string_or_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Some(vec![s.to_owned()]))
        }

        fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let result: Vec<String> =
                Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))?;
            Ok(Some(result))
        }
    }

    deserializer.deserialize_any(StringOrVec)
}

pub fn parse_key(input: &str) -> Result<Key, Box<dyn std::error::Error>> {
    let name = input.to_uppercase();

    // If evdev can parse this key
    if let Ok(key) = Key::from_str(&name) {
        return Ok(key);
    }
    if let Ok(key) = Key::from_str(&format!("KEY_{}", name)) {
        return Ok(key);
    }

    let key = match &name[..] {
        // Shift
        "SHIFT_R" => Key::KEY_RIGHTSHIFT,
        "SHIFT_L" => Key::KEY_LEFTSHIFT,
        // Control
        "CONTROL_R" => Key::KEY_RIGHTCTRL,
        "CONTROL_L" => Key::KEY_LEFTCTRL,
        "CTRL_R" => Key::KEY_RIGHTCTRL,
        "CTRL_L" => Key::KEY_LEFTCTRL,
        // Alt
        "ALT_R" => Key::KEY_RIGHTALT,
        "ALT_L" => Key::KEY_LEFTALT,
        // Windows
        "SUPER_R" => Key::KEY_RIGHTMETA,
        "SUPER_L" => Key::KEY_LEFTMETA,
        "WIN_R" => Key::KEY_RIGHTMETA,
        "WIN_L" => Key::KEY_LEFTMETA,
        // else
        _ => Key::KEY_RESERVED,
    };

    if key == Key::KEY_RESERVED {
        return Err(format!("Failed to parse key: '{}'", input).into());
    }

    Ok(key)
}

pub fn parse_modmap<'de, D>(deserializer: D) -> Result<Option<HashMap<Key, Key>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ModmapRemap;

    impl<'de> Visitor<'de> for ModmapRemap {
        type Value = Option<HashMap<Key, Key>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map from string to string")
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let remap: HashMap<String, String> =
                Deserialize::deserialize(value::MapAccessDeserializer::new(map))?;
            let mut modmap = HashMap::new();

            for (from, to) in remap.iter() {
                let from_key = parse_key(&from).map_err(M::Error::custom)?;
                let to_key = parse_key(&to).map_err(M::Error::custom)?;
                modmap.insert(from_key, to_key);
            }

            Ok(Some(modmap))
        }
    }

    deserializer.deserialize_any(ModmapRemap)
}

pub fn parse_key_combo(input: &str) -> Result<KeyCombo, Box<dyn std::error::Error>> {
    let keys: Vec<&str> = input.split("-").collect();
    if let Some((key, modifiers)) = keys.split_last() {
        let mut shift = false;
        let mut control = false;
        let mut alt = false;
        let mut windows = false;

        for modifier in modifiers.iter() {
            match parse_modifier(modifier) {
                Some(Modifier::Shift) => shift = true,
                Some(Modifier::Control) => control = true,
                Some(Modifier::Alt) => alt = true,
                Some(Modifier::Windows) => windows = true,
                None => {
                    return Err(format!("unknown modifier: {}", modifier).into());
                }
            }
        }

        Ok(KeyCombo {
            key: parse_key(key)?,
            shift,
            control,
            alt,
            windows,
        })
    } else {
        Err(format!("empty key_press: {}", input).into())
    }
}

pub fn parse_modifier(modifier: &str) -> Option<Modifier> {
    match &modifier.to_uppercase()[..] {
        // Shift
        "SHIFT" => Some(Modifier::Shift),
        // Control
        "C" => Some(Modifier::Control),
        "CTRL" => Some(Modifier::Control),
        "CONTROL" => Some(Modifier::Control),
        // Alt
        "M" => Some(Modifier::Alt),
        "META" => Some(Modifier::Alt),
        "ALT" => Some(Modifier::Alt),
        // Super
        "SUPER" => Some(Modifier::Windows),
        "WIN" => Some(Modifier::Windows),
        "WINDOWS" => Some(Modifier::Windows),
        _ => None,
    }
}
