use winit::{
    dpi::PhysicalPosition,
    event::ElementState,
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

    pub fn set_key_state(&mut self, logical_key: Key, state: ElementState) {
        let is_pressed = matches!(state, ElementState::Pressed);
        match logical_key {
            Key::Character(s) => self.set_character_key_state(s.as_str(), is_pressed),
            Key::Named(named_key) => self.set_named_key_state(named_key, is_pressed),
            _ => {}
        }
    }

    fn set_character_key_state(&mut self, key: &str, state: bool) {
        match key.to_lowercase().as_str() {
            "w" => self.key_w = state,
            "a" => self.key_a = state,
            "s" => self.key_s = state,
            "d" => self.key_d = state,
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
