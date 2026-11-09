use thiserror::Error;

pub const WHITE: u32 = 0xFFFFFF;
pub const CORNFLOWER_BLUE: u32 = 0x6495ED;

#[derive(Error, Debug)]
pub enum MeshError {
    #[error("Tried to build a mesh with no vertices.")]
    MissingVertices,

    #[error("Mesh index: {index} exceeded vertex count: {vertex_count}.")]
    IndexOutOfBounds { index: u16, vertex_count: usize },
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_srgb(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            r: Self::to_linear(red),
            g: Self::to_linear(green),
            b: Self::to_linear(blue),
            a: alpha,
        }
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self::from_srgb(r, g, b, 1.0)
    }

    fn to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::from_hex(WHITE)
    }
}

impl From<Color> for wgpu::Color {
    fn from(value: Color) -> wgpu::Color {
        wgpu::Color {
            r: value.r as f64,
            g: value.g as f64,
            b: value.b as f64,
            a: value.a as f64,
        }
    }
}

impl From<[f32; 4]> for Color {
    fn from(value: [f32; 4]) -> Self {
        Self::from_srgb(value[0], value[1], value[2], value[3])
    }
}
