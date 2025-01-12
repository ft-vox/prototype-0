use glam::Vec3;

pub struct Human {
    pub position: Vec3,
    pub velocity: Vec3,
    pub horizontal_rotation: f32,
    pub vertical_rotation: f32,
    pub is_jumping: bool,
    pub is_sprinting: bool,
    pub move_speed: MoveSpeed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MoveSpeed {
    Walk,
    Sprint,
    CreativeFly,
    FtMinecraftFly,
}

impl MoveSpeed {
    pub const fn speed_per_sec(&self) -> f32 {
        match self {
            Self::Walk => 4.317,
            Self::Sprint => 5.612,
            Self::CreativeFly => 10.89,
            Self::FtMinecraftFly => 40.00,
        }
    }
}

impl Human {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            horizontal_rotation: 0.0,
            vertical_rotation: 0.0,
            is_jumping: false,
            is_sprinting: false,
            move_speed: MoveSpeed::Walk,
        }
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        move_direction: [f32; 3],
        delta_horizontal_rotation: f32,
        delta_vertical_rotation: f32,
    ) {
        // Update rotation
        self.horizontal_rotation += delta_horizontal_rotation;
        self.horizontal_rotation %= 2.0 * std::f32::consts::PI;
        if self.horizontal_rotation < 0.0 {
            self.horizontal_rotation += 2.0 * std::f32::consts::PI;
        }

        self.vertical_rotation += delta_vertical_rotation;
        self.vertical_rotation = self.vertical_rotation.clamp(
            -0.4999 * std::f32::consts::PI,
            0.4999 * std::f32::consts::PI,
        );

        // Update movement
        let move_direction = {
            let move_direction = glam::Mat3::from_rotation_z(self.horizontal_rotation)
                * Vec3::new(move_direction[0], move_direction[1], move_direction[2]);
            let move_speed = move_direction.length();
            if move_speed > 1.0 {
                move_direction / move_speed
            } else {
                move_direction
            }
        };

        // Update position
        self.position += move_direction * self.move_speed.speed_per_sec() * delta_time;
    }

    pub fn get_eye_position(&self) -> Vec3 {
        self.position + Vec3::new(0.0, 0.0, 1.7) // Eye height is 1.7 blocks
    }

    pub fn get_eye_direction(&self) -> Vec3 {
        (glam::Mat3::from_rotation_z(self.horizontal_rotation)
            * glam::Mat3::from_rotation_x(self.vertical_rotation))
            * glam::Vec3::Y
    }

    pub fn toggle_sprint(&mut self) {
        self.is_sprinting = !self.is_sprinting;
        self.move_speed = if self.is_sprinting {
            MoveSpeed::Sprint
        } else {
            MoveSpeed::Walk
        };
    }
}
