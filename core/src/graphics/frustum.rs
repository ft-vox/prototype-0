use glam::{vec3, Mat4, Vec3};

pub struct Frustum {
    planes: [Plane; 6],
}

#[derive(Copy, Clone)]
struct Plane {
    normal: Vec3,
    distance: f32,
}

impl Plane {
    fn normalize(&mut self) {
        let length = self.normal.length();
        self.normal /= length;
        self.distance /= length;
    }
}

impl Frustum {
    pub fn new() -> Self {
        Frustum {
            planes: [Plane {
                normal: Vec3::new(0.0, 0.0, 0.0),
                distance: 0.0,
            }; 6],
        }
    }

    pub fn update(&mut self, view_projection: &Mat4) {
        let matrix_array = view_projection.to_cols_array_2d();
        self.planes[0].normal = vec3(
            matrix_array[0][3] + matrix_array[0][0],
            matrix_array[1][3] + matrix_array[1][0],
            matrix_array[2][3] + matrix_array[2][0],
        );
        self.planes[0].distance = matrix_array[3][3] + matrix_array[3][0];

        self.planes[1].normal = vec3(
            matrix_array[0][3] - matrix_array[0][0],
            matrix_array[1][3] - matrix_array[1][0],
            matrix_array[2][3] - matrix_array[2][0],
        );
        self.planes[1].distance = matrix_array[3][3] - matrix_array[3][0];

        self.planes[2].normal = vec3(
            matrix_array[0][3] + matrix_array[0][1],
            matrix_array[1][3] + matrix_array[1][1],
            matrix_array[2][3] + matrix_array[2][1],
        );
        self.planes[2].distance = matrix_array[3][3] + matrix_array[3][1];

        (&mut self.planes)[3].normal = vec3(
            matrix_array[0][3] - matrix_array[0][1],
            matrix_array[1][3] - matrix_array[1][1],
            matrix_array[2][3] - matrix_array[2][1],
        );
        self.planes[3].distance = matrix_array[3][3] - matrix_array[3][1];

        self.planes[4].normal = vec3(
            matrix_array[0][3] + matrix_array[0][2],
            matrix_array[1][3] + matrix_array[1][2],
            matrix_array[2][3] + matrix_array[2][2],
        );
        self.planes[4].distance = matrix_array[3][3] + matrix_array[3][2];

        self.planes[5].normal = vec3(
            matrix_array[0][3] - matrix_array[0][2],
            matrix_array[1][3] - matrix_array[1][2],
            matrix_array[2][3] - matrix_array[2][2],
        );
        self.planes[5].distance = matrix_array[3][3] - matrix_array[3][2];

        for plane in self.planes.iter_mut() {
            plane.normalize();
        }
    }

    pub fn is_sphere_in_frustum_planes(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let distance = plane.normal.dot(center) + plane.distance;
            if distance < -radius {
                return false;
            }
        }
        true
    }
}
