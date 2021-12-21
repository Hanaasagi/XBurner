use std::collections::HashMap;
use std::convert::From;
use std::error::Error;

use evdev::uinput::VirtualDevice;
use evdev::EventType;
use evdev::InputEvent;
use evdev::Key;
use log::debug;

use super::EventHandler;
use crate::config::Action;
use crate::config::Config;
use crate::config::KeyCombo;
use crate::config::Modifier;
use crate::executor::execute;
use crate::keycode::*;
use crate::notification::send_notify;
use crate::output::build_device;
use crate::x11::X11Client;
use crate::NAME;

// The value of InputEvent
const RELEASE: i32 = 0;
const PRESS: i32 = 1;
const REPEAT: i32 = 2;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum KeyState {
    PRESSED = PRESS as isize,
    RELEASED = RELEASE as isize,
    // REPEAT is ignored
}

impl From<bool> for KeyState {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            KeyState::PRESSED
        } else {
            KeyState::RELEASED
        }
    }
}

impl From<KeyState> for bool {
    #[inline]
    fn from(value: KeyState) -> Self {
        match value {
            KeyState::PRESSED => true,
            KeyState::RELEASED => false,
        }
    }
}

// TODO use trait
#[derive(Debug)]
struct Shift {
    left: KeyState,
    right: KeyState,
}

impl Default for Shift {
    fn default() -> Self {
        Self {
            left: KeyState::RELEASED,
            right: KeyState::RELEASED,
        }
    }
}

#[derive(Debug)]
struct Control {
    left: KeyState,
    right: KeyState,
}

impl Default for Control {
    fn default() -> Self {
        Self {
            left: KeyState::RELEASED,
            right: KeyState::RELEASED,
        }
    }
}

#[derive(Debug)]
struct Alt {
    left: KeyState,
    right: KeyState,
}
impl Default for Alt {
    fn default() -> Self {
        Self {
            left: KeyState::RELEASED,
            right: KeyState::RELEASED,
        }
    }
}

#[derive(Debug)]
struct Win {
    left: KeyState,
    right: KeyState,
}

impl Default for Win {
    fn default() -> Self {
        Self {
            left: KeyState::RELEASED,
            right: KeyState::RELEASED,
        }
    }
}

/// The inner struct for match keybinding.
#[derive(Debug)]
struct KeyMatchStruct {
    in_: Vec<String>,
    not_in: Vec<String>,
    action: Action,
}

pub struct DefaultEventHandler {
    // State
    shift: Shift,
    control: Control,
    alt: Alt,
    windows: Win,
    output_device: VirtualDevice,
    current_mode: Option<String>,
    all_modes: Vec<String>,
    switch_mode_keys: HashMap<KeyCombo, String>,
    cycle_switch_mode_key: Option<KeyCombo>,
    lookup_table: HashMap<String, HashMap<KeyCombo, KeyMatchStruct>>,
    x11_client: X11Client,
}

impl DefaultEventHandler {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let output_device =
            build_device().map_err(|e| format!("Failed to build an output device: {}", e))?;

        // Try to get the default mode
        let current_mode = config.options.clone().and_then(|x| x.default_mode);

        // Construct lookup table, O(1) HashMap is more faster for key matching.
        let (switch_mode_keys, lookup_table) = Self::construct_lookup_table(config.clone());
        let cycle_switch_mode_key = config.options.and_then(|x| x.mode_switch_key);

        let mut all_modes = vec![];
        if let Some(modes) = config.modes {
            for m in modes.keys().into_iter() {
                all_modes.push(m.to_owned());
            }
        }

        let x11_client = X11Client::new()?;

        Ok(Self {
            shift: Shift::default(),
            control: Control::default(),
            alt: Alt::default(),
            windows: Win::default(),
            output_device,
            current_mode,
            switch_mode_keys,
            lookup_table,
            cycle_switch_mode_key,
            all_modes,
            x11_client,
        })
    }

    fn construct_lookup_table(
        raw_config: Config,
    ) -> (
        HashMap<KeyCombo, String>,
        HashMap<String, HashMap<KeyCombo, KeyMatchStruct>>,
    ) {
        let mut res = HashMap::new();

        let mut switch_mode_keys: HashMap<KeyCombo, String> = HashMap::new();

        // check if we have modes
        if let Some(modes) = raw_config.modes {
            for (name, mode) in modes.iter() {
                // Construct switch mode key
                if let Some(combo) = &mode.switch_key {
                    switch_mode_keys.insert(combo.clone(), name.clone());
                }

                // Construct
                let mut groups = vec![];
                for group_name in mode.groups.iter() {
                    if let Some(g) = raw_config.groups.get(group_name) {
                        groups.push(g);
                    }
                }

                let mut kbs = HashMap::new();
                for g in groups.iter() {
                    for kb in g.key_bindings.iter() {
                        kbs.insert(
                            kb.key_combo.clone(),
                            KeyMatchStruct {
                                in_: g.in_.clone().unwrap_or_default(),
                                not_in: g.not_in.clone().unwrap_or_default(),
                                action: kb.get_action(),
                            },
                        );
                    }
                }

                res.insert(name.clone(), kbs);
            }
        } else {
            let mut kbs = HashMap::new();
            for g in raw_config.groups.values() {
                for kb in g.key_bindings.iter() {
                    kbs.insert(
                        kb.key_combo.clone(),
                        KeyMatchStruct {
                            in_: g.in_.clone().unwrap_or_default(),
                            not_in: g.not_in.clone().unwrap_or_default(),
                            action: kb.get_action(),
                        },
                    );
                }
            }

            res.insert(String::default(), kbs);
        }
        (switch_mode_keys, res)
    }

    /// Update state of modifier keys.
    fn update_modifier_state(&mut self, key: Key, state: KeyState) {
        match key {
            Key::KEY_LEFTSHIFT => {
                self.shift.left = state;
            }
            Key::KEY_RIGHTSHIFT => {
                self.shift.right = state;
            }
            Key::KEY_LEFTCTRL => {
                self.control.left = state;
            }
            Key::KEY_RIGHTCTRL => {
                self.control.right = state;
            }
            Key::KEY_LEFTALT => {
                self.alt.left = state;
            }
            Key::KEY_RIGHTALT => {
                self.alt.right = state;
            }
            Key::KEY_LEFTMETA => {
                self.windows.left = state;
            }
            Key::KEY_RIGHTMETA => {
                self.windows.right = state;
            }
            _ => {
                panic!("unexpected key {:?} at update_modifier_state", key);
            }
        };
    }

    fn get_expect_state(&self, modifier: Modifier, pressed: KeyState) -> (KeyState, KeyState) {
        let press_state = match modifier {
            Modifier::Shift => (self.shift.left, self.shift.right),
            Modifier::Control => (self.control.left, self.control.right),
            Modifier::Alt => (self.alt.left, self.alt.right),
            Modifier::Windows => (self.windows.left, self.windows.right),
        };

        if (bool::from(press_state.0) || bool::from(press_state.1)) == bool::from(pressed) {
            press_state.clone() // no change is needed
        } else if pressed == KeyState::PRESSED {
            // just press left
            (KeyState::PRESSED, KeyState::PRESSED)
        } else {
            // release all
            (KeyState::RELEASED, KeyState::RELEASED)
        }
    }

    /// Returns next_mode if we need to switch
    fn check_mode_switching(&self, key_combo: &KeyCombo) -> Option<String> {
        let mut next_mode = None;
        if let Some(current_mode) = &self.current_mode {
            if self.cycle_switch_mode_key.is_some()
                && key_combo == self.cycle_switch_mode_key.as_ref().unwrap()
            {
                let mut flag = false;
                // Make a cycle
                for (idx, mode) in self.all_modes.iter().cycle().enumerate() {
                    // Not found
                    if idx >= (self.all_modes.len() + 1) {
                        break;
                    }
                    // Found
                    if flag {
                        next_mode = Some(mode.to_string());
                        break;
                    }
                    if mode == current_mode {
                        flag = true;
                    }
                }
            }

            if next_mode.is_none() {
                next_mode = self.switch_mode_keys.get(&key_combo).map(|x| x.to_string());
            }
        }

        next_mode
    }

    fn dispatch_action(&mut self, action: &Action) -> Result<(), Box<dyn Error>> {
        debug!("Dispatch action => {:?}", action);

        match action {
            // Remap the key
            Action::Remap(key_press) => {
                let expect_shift = self.get_expect_state(Modifier::Shift, key_press.shift.into());
                let expect_control =
                    self.get_expect_state(Modifier::Control, key_press.control.into());
                let expect_alt = self.get_expect_state(Modifier::Alt, key_press.alt.into());
                let expect_windows =
                    self.get_expect_state(Modifier::Windows, key_press.windows.into());

                let prev_shift = self.send_modifier(Modifier::Shift, &expect_shift)?;
                let prev_control = self.send_modifier(Modifier::Control, &expect_control)?;
                let prev_alt = self.send_modifier(Modifier::Alt, &expect_alt)?;
                let prev_windows = self.send_modifier(Modifier::Windows, &expect_windows)?;

                self.send_key(&key_press.key, PRESS)?;
                self.send_key(&key_press.key, RELEASE)?;

                self.send_modifier(Modifier::Windows, &prev_windows)?;
                self.send_modifier(Modifier::Alt, &prev_alt)?;
                self.send_modifier(Modifier::Control, &prev_control)?;
                self.send_modifier(Modifier::Shift, &prev_shift)?;
            }
            // Execute shell command
            Action::Shell(command) => {
                let mut res = shlex::split(command).unwrap();
                let args = res.split_off(1);
                execute(res[0].clone(), args);
            }
        }
        Ok(())
    }

    fn send_modifier(
        &mut self,
        modifier: Modifier,
        desired: &(KeyState, KeyState),
    ) -> Result<(KeyState, KeyState), Box<dyn Error>> {
        debug!(
            "send modifier {:?}, {:?}, {:?}",
            modifier, self.control, desired
        );
        let mut current = match modifier {
            Modifier::Shift => (self.shift.left, self.shift.right),
            Modifier::Control => (self.control.left, self.control.right),
            Modifier::Alt => (self.alt.left, self.alt.right),
            Modifier::Windows => (self.windows.left, self.windows.right),
        }
        .clone();
        let original = current.clone();
        let left_key = match modifier {
            Modifier::Shift => Key::KEY_LEFTSHIFT,
            Modifier::Control => Key::KEY_LEFTCTRL,
            Modifier::Alt => Key::KEY_LEFTALT,
            Modifier::Windows => Key::KEY_LEFTMETA,
        };
        let right_key = match modifier {
            Modifier::Shift => Key::KEY_RIGHTSHIFT,
            Modifier::Control => Key::KEY_RIGHTCTRL,
            Modifier::Alt => Key::KEY_RIGHTALT,
            Modifier::Windows => Key::KEY_RIGHTMETA,
        };

        if !bool::from(current.0) && bool::from(desired.0) {
            self.send_key(&left_key, PRESS)?;
            current.0 = KeyState::PRESSED;
        } else if bool::from(current.0) && !bool::from(desired.0) {
            self.send_key(&left_key, RELEASE)?;
            current.0 = KeyState::RELEASED;
        }

        if !bool::from(current.1) && bool::from(desired.1) {
            self.send_key(&right_key, PRESS)?;
            current.1 = KeyState::PRESSED;
        } else if bool::from(current.1) && !bool::from(desired.1) {
            self.send_key(&right_key, RELEASE)?;
            current.1 = KeyState::RELEASED;
        }

        match modifier {
            Modifier::Shift => {
                self.shift = Shift {
                    left: current.0,
                    right: current.1,
                }
            }
            Modifier::Control => {
                self.control = Control {
                    left: current.0,
                    right: current.1,
                }
            }
            Modifier::Alt => {
                self.alt = Alt {
                    left: current.0,
                    right: current.1,
                }
            }
            Modifier::Windows => {
                self.windows = Win {
                    left: current.0,
                    right: current.1,
                }
            }
        };
        Ok(original)
    }

    fn find_action(
        &self,
        key_combo: &KeyCombo,
    ) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        // Find the action if we are using multi mode
        let current_mode = self.current_mode.clone().unwrap_or_default();
        let s = self
            .lookup_table
            .get(&current_mode)
            .and_then(|x| x.get(key_combo));

        if s.is_none() {
            return Ok(None);
        }

        let s = s.unwrap();

        // Check application name only if we have `in` and `notin` field
        if !s.in_.is_empty() || !s.not_in.is_empty() {
            let wm_class = self.x11_client.get_focus_window_wmclass()?;
            let class_name = std::str::from_utf8(wm_class.class())?.to_string();
            if !s.in_.is_empty() && !s.in_.contains(&class_name) {
                return Ok(None);
            }
            if !s.not_in.is_empty() && s.not_in.contains(&class_name) {
                return Ok(None);
            }
        }

        return Ok(Some(s.action.clone()));
    }

    fn send_key(&mut self, key: &Key, value: i32) -> std::io::Result<()> {
        let event = InputEvent::new(EventType::KEY, key.code(), value);
        self.send_event(event)
    }

    pub fn send_event(&mut self, event: InputEvent) -> std::io::Result<()> {
        self.output_device.emit(&[event])
    }
}

impl EventHandler for DefaultEventHandler {
    /// Processes the event and execute corresponding action. e.g. Shell, Remap
    fn handle_event(&mut self, event: InputEvent) -> Result<(), Box<dyn Error>> {
        // Just send the event we don't care.
        if event.event_type() != EventType::KEY {
            self.send_event(event)?;
            return Ok(());
        }
        debug!("Receive KEY event => {:?}", event);

        let key = Key::new(event.code());

        // The mapping of modifier keys is handled first, as it affects the matching later.
        if MODIFIER_KEYS.contains(&key) {
            let state = if event.value() == PRESS || event.value() == REPEAT {
                KeyState::PRESSED
            } else {
                KeyState::RELEASED
            };
            self.update_modifier_state(key, state);
            self.send_key(&key, event.value())?;
            return Ok(());
        }

        if event.value() == (KeyState::RELEASED as i32) {
            self.send_key(&key, event.value())?;
            return Ok(());
        }

        // So what key combo we pressed?
        let key_combo = KeyCombo {
            key: key.clone(),
            shift: bool::from(self.shift.left) || bool::from(self.shift.right),
            control: bool::from(self.control.left) || bool::from(self.control.right),
            alt: bool::from(self.alt.left) || bool::from(self.alt.right),
            windows: bool::from(self.windows.left) || bool::from(self.windows.right),
        };
        debug!("Current Key Combo => {:?}", key_combo);

        // Shall we switch to next mode?
        if let Some(next_mode) = self.check_mode_switching(&key_combo) {
            debug!(
                "Mode is switching from {:?} to {:?}",
                self.current_mode, next_mode
            );
            self.current_mode = Some(next_mode.to_string());
            send_notify(
                NAME,
                &format!("{} is switching to {} mode.", NAME, next_mode),
            )
            .ok();
            return Ok(());
        }

        // Find action and execute
        if let Some(action) = self.find_action(&key_combo)? {
            debug!("Find key binding action => {:?}", action);
            self.dispatch_action(&action)?;
            return Ok(());
        }

        // Make sure the event is sent, otherwise it will get stuck
        self.send_key(&key, event.value())?;
        Ok(())
    }
}
