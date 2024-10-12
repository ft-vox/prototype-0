pub struct Input {
    pub key_w: bool,
    pub key_a: bool,
    pub key_s: bool,
    pub key_d: bool,
    pub key_shift: bool,
    pub key_space: bool,
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
        }
    }
}
