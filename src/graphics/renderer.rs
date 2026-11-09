use super::{
    camera::Camera,
    mesh::MeshBuilder,
    object::{Material, Object3D, ObjectBuilder},
    pipeline::Shader,
    texture::Texture2D,
    types::Color,
};
use glam::Vec3;
use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,

    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,

    depth_texture: Texture2D,

    camera: Camera,
    object: Object3D,
    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let aspect_ratio = size.width as f32 / size.height as f32;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::from_env_or_default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
                apply_limit_buckets: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface = instance.create_surface(window.clone()).unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let voxel_mesh = MeshBuilder::build_voxel(&device);
        let depth_texture = Texture2D::new(
            Some("Depth Texture"),
            (size.width, size.height),
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
            &device,
        );

        let shader = Shader::load_from("assets/basic.wgsl", &device).unwrap();

        let camera = Camera::default()
            .with_eye(Vec3::new(0.0, 0.0, 4.0))
            .with_aspect_ratio(aspect_ratio);

        let object_texture =
            Texture2D::from_file("assets/new_texture.png", &device, &queue).unwrap();
        let object3d = ObjectBuilder::new(&voxel_mesh)
            .with_texture(&object_texture)
            .build(&device, &queue);

        let render_pipeline = ObjectBuilder::build_render_pipeline(&shader, &device);

        Self {
            device,
            queue,
            surface,
            surface_config,
            depth_texture,
            camera,
            object: object3d,
            render_pipeline,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.device, &self.surface_config);

            // recreate the depth texture to fit the new size
            self.depth_texture = Texture2D::new(
                Some("Depth Texture"),
                (size.width, size.height),
                wgpu::TextureFormat::Depth32Float,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
                &self.device,
            );
        }
    }

    pub fn render(&mut self, color: Color) {
        let wgpu::CurrentSurfaceTexture::Success(surface_texture) =
            self.surface.get_current_texture()
        else {
            return; // Skip the frame if it was not optimal for now
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });

        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Object3D Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color.into()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        self.object
            .draw(&mut render_pass, &self.camera, &self.queue);
        drop(render_pass);

        self.queue.submit(std::iter::once(command_encoder.finish()));
        self.queue.present(surface_texture);
    }
}
