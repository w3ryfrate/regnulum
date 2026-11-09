use super::types::MeshError;
use glam::{Vec2, Vec3};
use wgpu::util::DeviceExt;

/// Index values accepted by [`MeshBuilder::with_indices`].
///
/// This must match [`IndexData::FORMAT`]: `u16` values pair with `IndexFormat::Uint16`.
pub type Indices<'a> = &'a [u16];

#[rustfmt::skip]
const TRIANGLE_VERTICES: [Vertex; 3] = [
    Vertex::new(Vec3::new(-0.5,-0.5, 0.0),  Vec2::new(0.0, 1.0)),
    Vertex::new(Vec3::new(0.5, -0.5, 0.0),  Vec2::new(1.0, 1.0)),
    Vertex::new(Vec3::new(0.0, 0.5, 0.0),   Vec2::new(0.5, 0.0)),
];

#[rustfmt::skip]
const SQUARE_VERTICES: [Vertex; 4] = [
    Vertex::new(Vec3::new(-0.5,0.5, 0.0),   Vec2::new(0.0, 0.0)),
    Vertex::new(Vec3::new(-0.5,-0.5,0.0),   Vec2::new(0.0, 1.0)),
    Vertex::new(Vec3::new(0.5, -0.5,0.0),   Vec2::new(1.0, 1.0)),
    Vertex::new(Vec3::new(0.5, 0.5, 0.0),   Vec2::new(1.0, 0.0)),
];

#[rustfmt::skip]
const VOXEL_VERTICES: [Vertex; 24] = [
    // --- FRONT (Z = 0.5) ---
    Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec2::new(0.0, 0.0)), // 0
    Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec2::new(0.0, 1.0)), // 1
    Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec2::new(1.0, 1.0)), // 2
    Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec2::new(1.0, 0.0)), // 3

    // --- BACK (Z = -0.5) ---
    Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec2::new(0.0, 0.0)), // 4
    Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec2::new(0.0, 1.0)), // 5
    Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec2::new(1.0, 1.0)), // 6
    Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec2::new(1.0, 0.0)), // 7

    // --- LEFT (X = -0.5) ---
    Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec2::new(0.0, 0.0)), // 8
    Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec2::new(0.0, 1.0)), // 9
    Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec2::new(1.0, 1.0)), // 10
    Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec2::new(1.0, 0.0)), // 11

    // --- RIGHT (X = 0.5) ---
    Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec2::new(0.0, 0.0)), // 12
    Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec2::new(0.0, 1.0)), // 13
    Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec2::new(1.0, 1.0)), // 14
    Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec2::new(1.0, 0.0)), // 15

    // --- UP (Y = 0.5) ---
    Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec2::new(0.0, 0.0)), // 16
    Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec2::new(0.0, 1.0)), // 17
    Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec2::new(1.0, 1.0)), // 18
    Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec2::new(1.0, 0.0)), // 19

    // --- DOWN (Y = -0.5) ---
    Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec2::new(0.0, 0.0)), // 20
    Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec2::new(0.0, 1.0)), // 21
    Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec2::new(1.0, 1.0)), // 22
    Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec2::new(1.0, 0.0)), // 23
];

#[rustfmt::skip]
const VOXEL_INDICES: [u16; 36] = [
    0, 1, 2,   0, 2, 3,     // Front
    4, 5, 6,   4, 6, 7,     // Back
    8, 9, 10,  8, 10, 11,   // Left
    12, 13, 14, 12, 14, 15, // Right
    16, 17, 18, 16, 18, 19, // Up
    20, 21, 22, 20, 22, 23, // Down
];

#[rustfmt::skip]
const SQUARE_INDICES: [u16; 6] = [
    0, 1, 2, // first triangle
    0, 3, 2, // second triangle
];

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    position: Vec3,
    texture_uv: Vec2,
}

impl Vertex {
    pub const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
    };

    pub const fn new(pos: Vec3, uv: Vec2) -> Self {
        Self {
            position: pos,
            texture_uv: uv,
        }
    }
}

/// Builder for a [`Mesh`]. Borrows its input data, so the slices only need to
/// live until [`MeshBuilder::build`] is called.
pub struct MeshBuilder<'a> {
    vertices: &'a [Vertex],
    indices: Option<Indices<'a>>,
}

impl<'a> MeshBuilder<'a> {
    /// Starts a new mesh builder with a list of vertices.
    pub fn new(vertices: &'a [Vertex]) -> Self {
        Self {
            vertices: vertices,
            indices: None,
        }
    }

    /// Adds an index list, so vertices can be reused between triangles.
    pub fn with_indices(mut self, indices: Indices<'a>) -> Self {
        self.indices = Some(indices);
        self
    }

    /// Builds a mesh from the provided mesh data.
    ///
    /// # Errors
    ///
    /// - `MeshError::NoVertices` if the vertex slice is empty or hasn't been set.
    /// - `MeshError::IndexOutOfBounds` if any index refers to a vertex that doesn't exist.
    pub fn build(self, device: &wgpu::Device) -> Result<Mesh, MeshError> {
        if self.vertices.is_empty() {
            return Err(MeshError::MissingVertices);
        }

        // check immediately if the indices are valid
        if let Some(indices) = self.indices {
            if let Some(&index) = indices.iter().find(|&&i| i as usize >= self.vertices.len()) {
                return Err(MeshError::IndexOutOfBounds {
                    index,
                    vertex_count: self.vertices.len(),
                });
            }
        }

        let vertex_data = VertexData {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            count: self.vertices.len() as u32,
        };

        let index_data = self.indices.map(|indices| IndexData {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            count: indices.len() as u32,
        });

        Ok(Mesh {
            vertices: vertex_data,
            indices: index_data,
        })
    }

    /// Builds the built-in fallback square.
    ///
    /// Useful as a placeholder when loading another mesh fails.
    ///
    /// # Examples
    /// ```
    /// fn main() {
    ///    builder.build(&device).unwrap_or(MeshBuilder::build_square(&device))
    /// }
    /// ```
    pub fn build_square(device: &wgpu::Device) -> Mesh {
        Self::new(&SQUARE_VERTICES)
            .with_indices(&SQUARE_INDICES)
            .build(device)
            .expect("SQUARE_VERTICES is a constant, so building cannot fail.")
    }

    /// Builds a built-in triangle.
    pub fn build_triangle(device: &wgpu::Device) -> Mesh {
        Self::new(&TRIANGLE_VERTICES)
            .build(device)
            .expect("TRIANGLE_VERTICES is a constant, so building cannot fail.")
    }

    /// Builds a built-in voxel.
    pub fn build_voxel(device: &wgpu::Device) -> Mesh {
        Self::new(&VOXEL_VERTICES)
            .with_indices(&VOXEL_INDICES)
            .build(device)
            .expect("VOXEL_VERTICES is a constant, so building cannot fail.")
    }
}

/// Mandatory data attached to every [`Mesh`]: the vertices that make it up.
///
/// Fields are private so the only way to get one is through [`MeshBuilder::build`],
/// which enforces the "never empty" invariant.
#[derive(Clone)]
pub struct VertexData {
    buffer: wgpu::Buffer,

    /// How many vertices are in this `Mesh`.
    ///
    /// Can never be zero.
    count: u32,
}

impl VertexData {
    /// Gets the full [`wgpu::BufferSlice`] of the vertex buffer, ready for
    /// `wgpu::RenderPass`.
    pub fn buffer(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }

    /// Gets the count of vertices in this `Mesh`.
    pub fn count(&self) -> u32 {
        self.count
    }
}

/// Optional data attached to some [`Mesh`]es.
///
/// An index buffer lets triangles share vertices instead of repeating them,
/// which saves memory on meshes where the same vertex is used many times.
#[derive(Clone)]
pub struct IndexData {
    buffer: wgpu::Buffer,

    /// How many indices are in this `Mesh` (not how many triangles).
    count: u32,
}

impl IndexData {
    /// Format of every index in the buffer. Must match [`MeshIndices`].
    pub const FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

    /// Gets the full [`wgpu::BufferSlice`] of the index buffer, ready for
    /// `RenderPass::set_index_buffer`.
    pub fn buffer(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }

    /// Gets the count of indices in this `Mesh`.
    pub fn count(&self) -> u32 {
        self.count
    }
}

/// GPU-side data structure, contains vertices and how to draw them, optionally, via indices.
#[derive(Clone)]
pub struct Mesh {
    pub vertices: VertexData,
    pub indices: Option<IndexData>,
}

impl Mesh {
    /// Records the draw commands for this mesh into `pass`, choosing indexed
    /// or non-indexed drawing depending on whether `indices` is present.
    pub fn draw<'encoder>(&'encoder self, pass: &mut wgpu::RenderPass<'encoder>) {
        pass.set_vertex_buffer(0, self.vertices.buffer());

        match &self.indices {
            Some(indices) => {
                pass.set_index_buffer(indices.buffer(), IndexData::FORMAT);
                pass.draw_indexed(0..indices.count(), 0, 0..1);
            }
            None => pass.draw(0..self.vertices.count(), 0..1),
        }
    }
}
