use crate::*;

use image::DynamicImage;
use image::GenericImageView;
use image::ImageResult;

#[derive(Debug)]
pub struct TextureCreationParams<'a> {
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub mip_level_count: u32,
    pub filter_mode: wgpu::FilterMode,
    pub render_scale: f32,
    pub view_formats: &'a [wgpu::TextureFormat],
}

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat =
        wgpu::TextureFormat::Depth32Float;

    pub fn handle(&self) -> TextureHandle {
        TextureHandle::Raw(default_hash(&self.texture.global_id()))
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT |
                wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self { texture, view, sampler }
    }

    pub fn create_with_params(
        device: &wgpu::Device,
        params: TextureCreationParams,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: ((params.width as f32) * params.render_scale.sqrt()).round()
                as u32,
            height: ((params.height as f32) * params.render_scale.sqrt())
                .round() as u32,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&params.label),
            size,
            mip_level_count: params.mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // format: wgpu::TextureFormat::Rgba16Float,
            // format: wgpu::TextureFormat::Rgba8UnormSrgb,
            format: params.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING |
                wgpu::TextureUsages::COPY_DST |
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: params.view_formats,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{} View", params.label)),
            // TODO: fix this and move it to the pp layer instead
            mip_level_count: if params.mip_level_count > 0 {
                Some(1)
            } else {
                None
            },
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{} Sampler", params.label)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: params.filter_mode,
            min_filter: params.filter_mode,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            // size,
        }
    }

    pub fn create_scaled_mip_surface_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
        render_scale: f32,
        mip_level_count: u32,
        label: &str,
    ) -> Self {
        Self::create_scaled_mip_filter_surface_texture(
            device,
            config,
            format,
            render_scale,
            mip_level_count,
            wgpu::FilterMode::Linear,
            label,
        )
    }

    pub fn create_scaled_mip_filter_surface_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
        render_scale: f32,
        mip_level_count: u32,
        filter_mode: wgpu::FilterMode,
        label: &str,
    ) -> Self {
        Self::create_with_params(device, TextureCreationParams {
            label: label.to_string(),
            width: config.width,
            height: config.height,
            format,
            mip_level_count,
            filter_mode,
            render_scale,
            view_formats: &[],
        })
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
        is_normal_map: bool,
    ) -> ImageResult<(DynamicImage, Self)> {
        let img = image::load_from_memory(bytes)?;
        let tex =
            Self::from_image(device, queue, &img, Some(label), is_normal_map)?;

        Ok((img, tex))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        is_normal_map: bool,
    ) -> ImageResult<Self> {
        Self::from_image_ex(
            device,
            queue,
            img,
            label,
            is_normal_map,
            wgpu::AddressMode::Repeat,
        )
    }

    pub fn from_image_ex(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        is_normal_map: bool,
        address_mode: wgpu::AddressMode,
    ) -> ImageResult<Self> {
        let img = img.flipv();
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let format = if is_normal_map {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
            // wgpu::TextureFormat::Rgba8Unorm
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING |
                wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            // TODO: configure this
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self { texture, view, sampler })
    }

    pub fn from_image_uninit(
        device: &wgpu::Device,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> ImageResult<Self> {
        let dimensions = img.dimensions();
        Self::create_uninit(device, dimensions.0, dimensions.1, label)
    }

    pub fn create_uninit(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        label: Option<&str>,
    ) -> ImageResult<Self> {
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING |
                wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self { texture, view, sampler })
    }
}
