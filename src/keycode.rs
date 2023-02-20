use std::collections::HashSet;

use evdev::Key;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MODIFIER_KEYS: HashSet<Key> = {
        HashSet::from_iter([
            Key::KEY_LEFTSHIFT,
            Key::KEY_RIGHTSHIFT,
            Key::KEY_LEFTCTRL,
            Key::KEY_RIGHTCTRL,
            Key::KEY_LEFTALT,
            Key::KEY_RIGHTALT,
            Key::KEY_LEFTMETA,
            Key::KEY_RIGHTMETA,
        ].into_iter())
    };
    pub static ref SHIFT_KEYS: HashSet<Key> =
        HashSet::from_iter([Key::KEY_LEFTSHIFT, Key::KEY_RIGHTSHIFT,].into_iter());
    pub static ref CONTROL_KEYS: HashSet<Key> =
        HashSet::from_iter([Key::KEY_LEFTCTRL, Key::KEY_RIGHTCTRL,].into_iter());
    pub static ref ALT_KEYS: HashSet<Key> = {
        HashSet::from_iter([
            Key::new(Key::KEY_LEFTALT.code()),
            Key::new(Key::KEY_RIGHTALT.code()),
        ].into_iter())
    };
    pub static ref WINDOWS_KEYS: HashSet<Key> =
        HashSet::from_iter([Key::KEY_LEFTMETA, Key::KEY_RIGHTMETA,].into_iter());
}
