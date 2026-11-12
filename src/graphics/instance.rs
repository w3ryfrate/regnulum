use super::{
    camera::Camera,
    mesh::Mesh,
    pipeline::{RenderPipelineBuilder, Shader},
    texture::Texture2D,
    types::Color,
};
use glam::{Mat4, Quat, Vec3, Vec3A};
use wgpu::util::DeviceExt;

pub struct ObjectBuilder {
    mesh: Mesh,
    texture: Option<Texture2D>,
    transform: Option<Transform>,
    material: Option<Material>,
}

impl ObjectBuilder {
    pub fn new(mesh: &Mesh) -> Self {
        Self {
            mesh: mesh.clone(),
            transform: None,
            material: None,
            texture: None,
        }
    }

    pub fn build_render_pipeline(shader: &Shader, device: &wgpu::Device) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Object3D Render Pipeline Layout"),
            bind_group_layouts: &[Some(&Self::bind_group_layout(device))],
            immediate_size: 0,
        });

        RenderPipelineBuilder::new(shader)
            .with_label("Object3D Render Pipeline")
            .with_layout(&pipeline_layout)
            .with_depth_stencil(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            })
            .build(&device)
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    pub fn with_material(mut self, material: Material) -> Self {
        self.material = Some(material);
        self
    }

    pub fn with_texture(mut self, texture: &Texture2D) -> Self {
        self.texture = Some(texture.clone());
        self
    }

    pub fn build(self, device: &wgpu::Device, queue: &wgpu::Queue) -> Object3D {
        let transform = self.transform.unwrap_or(Transform::IDENTITY);
        let material = self.material.unwrap_or_default();
        let texture = self
            .texture
            .unwrap_or_else(|| Texture2D::white(device, queue));

        let mut buffer_data = Vec::with_capacity(Object3D::UNIFORMS_SIZE as usize);
        buffer_data.extend_from_slice(bytemuck::bytes_of(&transform.model().to_cols_array()));
        buffer_data.extend_from_slice(bytemuck::bytes_of(&material));

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Object3D Buffer"),
            contents: &buffer_data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Object3D Bind Group"),
            layout: &Self::bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler()),
                },
            ],
        });

        Object3D {
            transform,
            material,
            bind_group,
            mesh: self.mesh,
            buffer,
        }
    }

    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Object3D Bind Group Layout"),
            entries: &[
                // uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(Object3D::UNIFORMS_SIZE),
                    },
                    count: None,
                },
                // t_diffuse
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // s_diffuse
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub position: Vec3A,
    pub rotation: Quat,
    pub scale: Vec3A,
}

impl Transform {
    const IDENTITY: Self = Self {
        position: Vec3A::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3A::ONE,
    };

    pub fn new(pos: Vec3, rot: Quat, scale: Vec3) -> Self {
        Self {
            position: pos.into(),
            rotation: rot,
            scale: scale.into(),
        }
    }

    pub fn with_position(mut self, pos: Vec3) -> Self {
        self.position = pos.into();
        self
    }

    pub fn model(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale.into(),
            self.rotation,
            self.position.into(),
        )
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    pub color: Color,
}

pub struct Object3D {
    pub transform: Transform,

    mesh: Mesh,
    material: Material,

    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

impl Object3D {
    const UNIFORMS_SIZE: u64 = (size_of::<Mat4>() + size_of::<Material>()) as u64;

    pub fn draw<'encoder>(
        &'encoder self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        camera: &Camera,
        queue: &wgpu::Queue,
    ) {
        let transform = camera.build_transform(self.transform.model());

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[transform]));

        render_pass.set_bind_group(0, &self.bind_group, &[]);
        self.mesh.draw(render_pass);
    }
}
