use std::array::IntoIter;
use std::collections::HashSet;

use evdev::Key;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MODIFIER_KEYS: HashSet<Key> = {
        HashSet::from_iter(IntoIter::new([
            Key::KEY_LEFTSHIFT,
            Key::KEY_RIGHTSHIFT,
            Key::KEY_LEFTCTRL,
            Key::KEY_RIGHTCTRL,
            Key::KEY_LEFTALT,
            Key::KEY_RIGHTALT,
            Key::KEY_LEFTMETA,
            Key::KEY_RIGHTMETA,
        ]))
    };
    pub static ref SHIFT_KEYS: HashSet<Key> =
        HashSet::from_iter(IntoIter::new([Key::KEY_LEFTSHIFT, Key::KEY_RIGHTSHIFT,]));
    pub static ref CONTROL_KEYS: HashSet<Key> =
        HashSet::from_iter(IntoIter::new([Key::KEY_LEFTCTRL, Key::KEY_RIGHTCTRL,]));
    pub static ref ALT_KEYS: HashSet<Key> = {
        HashSet::from_iter(IntoIter::new([
            Key::new(Key::KEY_LEFTALT.code()),
            Key::new(Key::KEY_RIGHTALT.code()),
        ]))
    };
    pub static ref WINDOWS_KEYS: HashSet<Key> =
        HashSet::from_iter(IntoIter::new([Key::KEY_LEFTMETA, Key::KEY_RIGHTMETA,]));
}
