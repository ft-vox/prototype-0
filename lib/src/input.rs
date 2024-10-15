use winit::{
    dpi::PhysicalPosition,
    event::ElementState,
    keyboard::{Key, NamedKey},
};

pub struct InputState {
    pub key_w: bool,
    pub key_a: bool,
    pub key_s: bool,
    pub key_d: bool,
    pub key_shift: bool,
    pub key_space: bool,
    pub key_esc: bool,
    pub key_tab: bool,
}

impl InputState {
    fn new() -> Self {
        Self {
            key_w: false,
            key_a: false,
            key_s: false,
            key_d: false,
            key_shift: false,
            key_space: false,
            key_esc: false,
            key_tab: false,
        }
    }
}

pub struct EventDrivenInput {
    pub input_state: InputState,
    pub local_cursor_position: PhysicalPosition<f64>,
}

impl EventDrivenInput {
    pub fn new() -> Self {
        Self {
            input_state: InputState::new(),
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
            "w" => self.input_state.key_w = state,
            "a" => self.input_state.key_a = state,
            "s" => self.input_state.key_s = state,
            "d" => self.input_state.key_d = state,
            _ => {}
        }
    }

    fn set_named_key_state(&mut self, key: NamedKey, state: bool) {
        match key {
            NamedKey::Shift => self.input_state.key_shift = state,
            NamedKey::Space => self.input_state.key_space = state,
            NamedKey::Escape => self.input_state.key_esc = state,
            NamedKey::Tab => self.input_state.key_tab = state,
            _ => {}
        }
    }
}

pub struct FrameDrivenInput {
    pub key_pressed: InputState,
    pub key_down: InputState,
    pub key_up: InputState,
    pub local_cursor_position: PhysicalPosition<f64>,
}

impl FrameDrivenInput {}
