use glam::{Mat4, Vec3};

const DEFAULT_ASPECT_RATIO: f32 = 16.0 / 9.0;

pub struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn build_transform(&self, model: Mat4) -> Mat4 {
        let view = glam::camera::rh::view::look_to_mat4(
            self.eye,
            (self.target - self.eye).normalize(),
            self.up,
        );

        let proj = glam::camera::rh::proj::directx::perspective(
            f32::to_radians(self.fovy),
            self.aspect,
            self.znear,
            self.zfar,
        );

        proj * view * model
    }

    pub fn with_eye(mut self, eye: Vec3) -> Self {
        self.eye = eye;
        self
    }

    pub fn with_aspect_ratio(mut self, aspect_ratio: f32) -> Self {
        self.aspect = aspect_ratio;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: Default::default(),
            target: glam::Vec3::NEG_Z,
            up: glam::Vec3::Y,
            aspect: DEFAULT_ASPECT_RATIO, // 16:9
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}
