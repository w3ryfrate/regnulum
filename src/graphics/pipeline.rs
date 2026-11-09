use super::mesh::Vertex;

const COLOR_TARGET_STATE: wgpu::ColorTargetState = wgpu::ColorTargetState {
    format: wgpu::TextureFormat::Rgba8UnormSrgb,
    blend: None,
    write_mask: wgpu::ColorWrites::all(),
};

pub struct Shader {
    module: wgpu::ShaderModule,
    has_fragment: bool,
}

impl Shader {
    pub fn load_from(path: &str, device: &wgpu::Device) -> Result<Self, std::io::Error> {
        let source = std::fs::read_to_string(path)?;
        Ok(Self {
            module: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(path),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            }),
            has_fragment: !path.contains(".vert"),
        })
    }
}

struct ShaderState<'a>(wgpu::VertexState<'a>, Option<wgpu::FragmentState<'a>>);

pub struct RenderPipelineBuilder<'a> {
    label: Option<&'a str>,
    layout: Option<&'a wgpu::PipelineLayout>,
    shader: ShaderState<'a>,
    primitive: wgpu::PrimitiveState,
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
    multiview: Option<std::num::NonZeroU32>,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(shader: &'a Shader) -> Self {
        let shader_state = ShaderState(
            wgpu::VertexState {
                module: &shader.module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(Vertex::BUFFER_LAYOUT)],
            },
            if shader.has_fragment {
                Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(COLOR_TARGET_STATE)],
                })
            } else {
                None
            },
        );

        Self {
            label: None,
            layout: None,
            shader: shader_state,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        }
    }

    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_layout(mut self, layout: &'a wgpu::PipelineLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn with_depth_stencil(mut self, depth_stencil: wgpu::DepthStencilState) -> Self {
        self.depth_stencil = Some(depth_stencil);
        self
    }

    pub fn build(self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.label,
            layout: self.layout,
            vertex: self.shader.0,
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            fragment: self.shader.1,
            multiview_mask: self.multiview,
            cache: None,
        })
    }
}
