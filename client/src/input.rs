use std::collections::HashMap;
use winit::{
    dpi::PhysicalPosition,
    event::ElementState,
    keyboard::{Key, NamedKey, SmolStr},
};

/// EventDrivenInput will reset key state when input events occur.
pub struct EventDrivenInput {
    pub key_pressed: HashMap<Key, bool>,
    pub local_cursor_position: PhysicalPosition<f64>,
}

impl EventDrivenInput {
    pub fn new() -> Self {
        Self {
            key_pressed: HashMap::new(),
            local_cursor_position: PhysicalPosition::new(0.0, 0.0),
        }
    }

    pub fn set_key_state(&mut self, logical_key: Key, state: ElementState) {
        let is_pressed = matches!(state, ElementState::Pressed);
        let key_to_insert = match logical_key {
            Key::Character(ref s) => {
                let lowercase_key = SmolStr::new(s.to_lowercase());
                Key::Character(lowercase_key)
            }
            _ => logical_key,
        };
        self.key_pressed.insert(key_to_insert, is_pressed);
    }

    pub fn set_cursor_position(&mut self, position: PhysicalPosition<f64>) {
        self.local_cursor_position = position;
    }
}

/// FrameDrivenInput will update input states every frame.
/// Added up & down state for game logic.
pub struct FrameDrivenInput {
    pub key_pressed: HashMap<Key, bool>,
    pub key_down: HashMap<Key, bool>,
    pub key_up: HashMap<Key, bool>,
    pub local_cursor_position: PhysicalPosition<f64>,
}

impl FrameDrivenInput {
    pub fn new() -> Self {
        Self {
            key_pressed: HashMap::new(),
            key_down: HashMap::new(),
            key_up: HashMap::new(),
            local_cursor_position: PhysicalPosition::new(0.0, 0.0),
        }
    }

    pub fn update(&mut self, event_driven_input: &EventDrivenInput) {
        let previous_key_pressed = self.key_pressed.clone();
        self.key_pressed = event_driven_input.key_pressed.clone();

        self.key_down.clear();
        self.key_up.clear();

        for (key, &is_pressed) in &self.key_pressed {
            let was_pressed = previous_key_pressed.get(key).cloned().unwrap_or(false);
            if is_pressed && !was_pressed {
                self.key_down.insert(key.clone(), true);
            }
            if !is_pressed && was_pressed {
                self.key_up.insert(key.clone(), true);
            }
        }

        self.local_cursor_position = event_driven_input.local_cursor_position;
    }

    pub fn get_key_pressed(&self, str: &str) -> bool {
        *self.key_pressed.get(&str_to_key(str)).unwrap_or(&false)
    }

    pub fn get_key_down(&self, str: &str) -> bool {
        *self.key_down.get(&str_to_key(str)).unwrap_or(&false)
    }

    pub fn _get_key_up(&self, str: &str) -> bool {
        *self.key_up.get(&str_to_key(str)).unwrap_or(&false)
    }
}

fn str_to_key(str: &str) -> Key {
    match str {
        "space" => Key::Named(NamedKey::Space),
        "shift" => Key::Named(NamedKey::Shift),
        "tab" => Key::Named(NamedKey::Tab),
        "esc" => Key::Named(NamedKey::Escape),
        "ctrl" => Key::Named(NamedKey::Control),
        _ => Key::Character(SmolStr::new(str)),
    }
}
