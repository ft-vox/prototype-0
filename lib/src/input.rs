use winit::{
    dpi::PhysicalPosition,
    keyboard::{Key, NamedKey},
};

pub struct Input {
    pub key_w: bool,
    pub key_a: bool,
    pub key_s: bool,
    pub key_d: bool,
    pub key_shift: bool,
    pub key_space: bool,
    pub local_cursor_position: PhysicalPosition<f64>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            key_w: false,
            key_a: false,
            key_s: false,
            key_d: false,
            key_shift: false,
            key_space: false,
            local_cursor_position: PhysicalPosition::new(0.0, 0.0),
        }
    }

    pub fn set_key_pressed(&mut self, logical_key: Key) {
        self.set_key_state(logical_key, true);
    }

    pub fn set_key_released(&mut self, logical_key: Key) {
        self.set_key_state(logical_key, false);
    }

    fn set_key_state(&mut self, logical_key: Key, state: bool) {
        match logical_key {
            Key::Character(s) => {
                self.set_character_key_state(s.as_str(), state);
            }
            Key::Named(named_key) => {
                self.set_named_key_state(named_key, state);
            }
            _ => {}
        }
    }

    fn set_character_key_state(&mut self, key: &str, state: bool) {
        match key {
            "w" | "W" => self.key_w = state,
            "a" | "A" => self.key_a = state,
            "s" | "S" => self.key_s = state,
            "d" | "D" => self.key_d = state,
            _ => {}
        }
    }

    fn set_named_key_state(&mut self, key: NamedKey, state: bool) {
        match key {
            NamedKey::Shift => self.key_shift = state,
            NamedKey::Space => self.key_space = state,
            _ => {}
        }
    }
}
