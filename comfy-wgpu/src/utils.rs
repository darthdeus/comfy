use crate::*;

pub const FRAG_SHADER_PREFIX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/frag-shader-prefix.wgsl"
));

pub const CAMERA_BIND_GROUP_PREFIX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/camera-bind-group.wgsl"
));

pub const SHADER_POST_PROCESSING_VERTEX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/post_processing_vertex.wgsl"
));

pub const COPY_SHADER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/copy.wgsl"));

#[macro_export]
macro_rules! engine_shader_source {
    ($name:literal) => {{
        let shader = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/shaders/",
            $name,
            ".wgsl"
        ));

        shader
    }};
}

pub fn sprite_shader_from_fragment(source: &str) -> String {
    format!("{}{}{}", CAMERA_BIND_GROUP_PREFIX, FRAG_SHADER_PREFIX, source)
}

pub fn post_process_shader_from_fragment(source: &str) -> String {
    format!(
        "{}{}{}",
        CAMERA_BIND_GROUP_PREFIX, SHADER_POST_PROCESSING_VERTEX, source
    )
}

#[macro_export]
macro_rules! create_engine_post_processing_shader {
    ($shaders:expr, $name:literal) => {{
        let full_shader =
            post_process_shader_from_fragment(engine_shader_source!($name));

        let shader_id =
            create_shader($shaders, $name, &full_shader, HashMap::new())
                .unwrap();

        $shaders.get(shader_id).unwrap().clone()
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
    let texture = Texture::from_image_ex(
        &context.device,
        &context.queue,
        &img,
        Some(name),
        false,
        address_mode,
    )
    .unwrap();

    let bind_group = context.device.simple_bind_group(
        Some(&format!("{}_bind_group", name)),
        &texture,
        &context.texture_layout,
    );

    ASSETS.borrow_mut().insert_handle(name, handle);
    ASSETS.borrow_mut().texture_image_map.lock().insert(handle, img);
    textures.insert(handle, BindableTexture { bind_group, texture });
}

pub struct MipmapGenerator {
    pub format: wgpu::TextureFormat,
    pub blit_pipeline: wgpu::RenderPipeline,
    pub blit_layout: wgpu::BindGroupLayout,
}

impl MipmapGenerator {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let blit_pipeline = {
            // TODO: unify with other shaders
            let shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Blit Shader"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                        engine_shader_source!("blit"),
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
