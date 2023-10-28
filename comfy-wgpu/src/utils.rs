use crate::*;

pub const SHADER_STRUCTS_PREFIX: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/structs.wgsl"));

pub const SHADER_POST_PROCESSING_VERTEX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/post_processing_vertex.wgsl"
));

#[macro_export]
macro_rules! reloadable_wgsl_shader {
    ($name:literal) => {{
        cfg_if! {
            if #[cfg(any(feature = "ci-release", target_arch = "wasm32"))] {
                let shader = include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/shaders/", $name, ".wgsl"));
            } else {
                let path = concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/shaders/", $name, ".wgsl"
                );

                info!("DEV loading shader: {}", path);
                let shader: String = std::fs::read_to_string(path).unwrap().into();
            }
        }

        Shader {
            name: format!("{} Shader", $name),
            source: format!("{}{}", SHADER_STRUCTS_PREFIX, shader).into(),
        }
    }};
}

// #[macro_export]
// macro_rules! include_wgsl_fragment_shader {
//     ($name:expr, $source:expr) => {{
//         Shader { name: $name.to_string(), source: full_shader.to_string() }
//     }};
// }

#[macro_export]
macro_rules! reloadable_wgsl_fragment_shader {
    ($name:literal) => {{
        let frag_shader_prefix = format!(
            "{}{}",
            SHADER_STRUCTS_PREFIX, SHADER_POST_PROCESSING_VERTEX
        );

        cfg_if! {
            if #[cfg(any(feature = "ci-release", target_arch = "wasm32"))] {
                let frag_part =
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/shaders/", $name, ".wgsl"));
            } else {
                let path = concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/shaders/", $name, ".wgsl");
                info!("DEV loading shader: {}", path);

                let frag_part = std::fs::read_to_string(path)
                    .expect(&format!("shader at {path} must exist"));
            }
        }

        let full_shader = format!("{}{}", frag_shader_prefix, frag_part);

        Shader { name: $name.to_string(), source: full_shader.to_string() }
    }};
}

pub fn load_texture_from_engine_bytes(
    context: &GraphicsContext,
    name: &str,
    bytes: &[u8],
    textures: &mut TextureMap,
    address_mode: wgpu::AddressMode,
) {
    let handle = texture_path(name);

    let img = image::load_from_memory(bytes).expect("must be valid image");
    let error_texture = Texture::from_image_ex(
        &context.device,
        &context.queue,
        &img,
        Some(name),
        false,
        address_mode,
    )
    .unwrap();

    let error_bind_group = context.device.simple_bind_group(
        &format!("{}_bind_group", name),
        &error_texture,
        &context.texture_layout,
    );

    ASSETS.borrow_mut().insert_handle(name, handle);
    ASSETS.borrow_mut().texture_image_map.lock().insert(handle, img);
    textures.insert(handle, (error_bind_group, error_texture));
}

pub fn simple_fragment_shader(
    name: &'static str,
    frag: &'static str,
) -> Shader {
    Shader {
        name: name.to_string(),
        source: format!(
            "{}{}{}",
            SHADER_STRUCTS_PREFIX, SHADER_POST_PROCESSING_VERTEX, frag
        ),
    }
}

pub struct MipmapGenerator {
    pub format: wgpu::TextureFormat,
    pub blit_pipeline: wgpu::RenderPipeline,
    pub blit_layout: wgpu::BindGroupLayout,
}

impl MipmapGenerator {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let blit_pipeline = {
            let shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Blit Shader"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                        include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/shaders/blit.wgsl"
                        )),
                    )),
                });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Blit Render Pipeline"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            })
        };

        let blit_layout = blit_pipeline.get_bind_group_layout(0);

        Self { format, blit_pipeline, blit_layout }
    }

    pub fn generate_mipmaps(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        texture: &wgpu::Texture,
        mip_count: u32,
    ) {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Mip Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let views = (0..mip_count)
            .map(|mip| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Mip View"),
                    format: None,
                    dimension: None,
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: mip,
                    mip_level_count: Some(1),
                    base_array_layer: 0,
                    array_layer_count: None,
                })
            })
            .collect::<Vec<_>>();

        for target_mip in 1..mip_count as usize {
            let bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.blit_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &views[target_mip - 1],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                    label: None,
                });

            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &views[target_mip],
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                });

            rpass.set_pipeline(&self.blit_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }
    }
}
