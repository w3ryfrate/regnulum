use wgpu::util::DeviceExt;

const SAMPLER_SETTINGS: &wgpu::SamplerDescriptor = &wgpu::SamplerDescriptor {
    label: None,
    address_mode_u: wgpu::AddressMode::ClampToEdge,
    address_mode_v: wgpu::AddressMode::ClampToEdge,
    mag_filter: wgpu::FilterMode::Nearest,
    min_filter: wgpu::FilterMode::Nearest,
    address_mode_w: wgpu::AddressMode::ClampToEdge,
    mipmap_filter: wgpu::MipmapFilterMode::Nearest,
    lod_min_clamp: 0.0,
    lod_max_clamp: 32.0,
    compare: None,
    anisotropy_clamp: 1,
    border_color: None,
};

/// A texture in two dimensions, commonly used for rendering graphics assets.
#[derive(Clone)]
pub struct Texture2D {
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Texture2D {
    /// Creates a new empty texture.
    pub fn new(
        label: Option<&str>,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        device: &wgpu::Device,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(SAMPLER_SETTINGS);

        Self { view, sampler }
    }

    /// Creates a 1x1 opaque white texture.
    ///
    /// Multiplying by its sample is a no-op, so it can be bound wherever a shader
    /// expects a texture but the geometry should be drawn untextured.
    pub fn white(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_rgba8(
            Some("White Texture"),
            (1, 1),
            &[255, 255, 255, 255],
            device,
            queue,
        )
    }

    /// Creates a texture from raw, tightly packed RGBA8 pixel data.
    pub fn from_rgba8(
        label: Option<&str>,
        size: (u32, u32),
        rgba: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            rgba,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(SAMPLER_SETTINGS);

        Self { view, sampler }
    }

    /// Creates a new texture from the provided slice of bytes `&[u8]`.
    ///
    /// This function assumes the format to be [`wgpu::TextureFormat::Rgba8UnormSrgb`] and the usage to
    /// be [`wgpu::TextureUsages::TEXTURE_BINDING`], which are commonly used for rendering.
    ///
    /// # Errors
    /// - [`image::ImageError`] if loading or reading the bytes fails.
    pub fn from_bytes(
        label: Option<&str>,
        bytes: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self, image::ImageError> {
        let image = image::load_from_memory(&bytes)?;
        let rgba = image.to_rgba8();
        let dimensions = rgba.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label,
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &rgba,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(SAMPLER_SETTINGS);

        Ok(Self {
            view: texture_view,
            sampler,
        })
    }

    /// Loads a new texture, reading the bytes of the file at the provided absolute path.
    ///
    /// This function assumes the format to be [`wgpu::TextureFormat::Rgba8UnormSrgb`] and the usage to
    /// be [`wgpu::TextureUsages::TEXTURE_BINDING`], which are commonly used for rendering.
    ///
    /// # Errors
    /// - [`image::ImageError`] if loading or reading the bytes fails.
    pub fn from_file(
        path: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self, image::ImageError> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(Some(path), &bytes, device, queue)
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}
