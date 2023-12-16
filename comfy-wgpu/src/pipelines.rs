use std::sync::atomic::AtomicU32;

use crate::*;

static GENERATED_RENDER_TARGET_IDS: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, Debug)]
pub struct RenderTargetParams {
    pub label: String,
    pub size: UVec2,
    pub filter_mode: wgpu::FilterMode,
}

/// Creates a new render target with given dimensions. Among other parameters a label is
/// required so that graphic debuggers like RenderDoc can display its name properly.
pub fn create_render_target(
    renderer: &mut WgpuRenderer,
    params: &RenderTargetParams,
) -> RenderTargetId {
    let id = gen_render_target();

    let c = &renderer.context;

    let size = wgpu::Extent3d {
        width: params.size.x,
        height: params.size.y,
        depth_or_array_layers: 1,
    };

    let texture = c.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&params.label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING |
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(&format!("{} View", params.label)),
        format: None,
        dimension: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });

    let sampler = c.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&format!("{} Sampler", params.label)),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: params.filter_mode,
        min_filter: params.filter_mode,
        mipmap_filter: params.filter_mode,
        ..Default::default()
    });

    let bind_group = c.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("{} Bind Group", params.label)),
        layout: &c.texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    renderer.render_targets.borrow_mut().insert(id, UserRenderTarget {
        creation_params: params.clone(),
        texture,
        view,
        sampler,
        bind_group,
    });

    id
}

pub struct UserRenderTarget {
    pub creation_params: RenderTargetParams,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

/// Allocates a new render target id
fn gen_render_target() -> RenderTargetId {
    let id = GENERATED_RENDER_TARGET_IDS
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    RenderTargetId(id)
}

pub fn ensure_pipeline_exists(
    c: &mut WgpuRenderer,
    pass_data: &MeshDrawData,
    sprite_shader_id: ShaderId,
) -> String {
    let shaders = c.shaders.borrow();

    let maybe_shader_instance_id = pass_data.shader;

    let maybe_shader =
        maybe_shader_instance_id.as_ref().and_then(|instance_id| {
            let instance = get_shader_instance(*instance_id);
            shaders.get(instance.id)
        });

    let name = format!(
        "{} {:?} {:?} {:?}",
        if maybe_shader_instance_id.is_some() {
            "USER(Mesh)"
        } else {
            "BUILTIN(Mesh)"
        },
        pass_data.blend_mode,
        maybe_shader,
        c.enable_z_buffer
    );

    let mesh_pipeline = if let Some(shader) = maybe_shader {
        RenderPipeline::User(
            c.user_pipelines.entry(name.clone()).or_insert_with(|| {
                create_user_pipeline(
                    &name,
                    pass_data,
                    shader,
                    &c.context,
                    &c.texture_layout,
                    &c.camera_bind_group_layout,
                    c.enable_z_buffer,
                )
            }),
        )
    } else {
        RenderPipeline::Wgpu(c.pipelines.entry(name.clone()).or_insert_with(
            || {
                create_render_pipeline_with_layout(
                    &name,
                    &c.context.device,
                    wgpu::TextureFormat::Rgba16Float,
                    &[&c.texture_layout, &c.camera_bind_group_layout],
                    &[SpriteVertex::desc()],
                    shaders.get(sprite_shader_id).unwrap(),
                    pass_data.blend_mode,
                    c.enable_z_buffer,
                )
                .unwrap()
            },
        ))
    };

    if let RenderPipeline::User(user_pipeline) = mesh_pipeline {
        if let Some(shader_instance_id) = maybe_shader_instance_id {
            let shader_instance = get_shader_instance(shader_instance_id);
            let shader = shaders.get(shader_instance.id).unwrap();

            for (buffer_name, buffer) in
                user_pipeline.buffers.iter().sorted_by_key(|x| x.0)
            {
                if let Some(Uniform::F32(OrderedFloat(value))) =
                    shader_instance.uniforms.get(buffer_name)
                {
                    c.context.queue.write_buffer(
                        buffer,
                        0,
                        bytemuck::cast_slice(&[*value]),
                    );
                } else if let UniformDef::F32(Some(default_value)) =
                    shader.uniform_defs.get(buffer_name).unwrap()
                {
                    c.context.queue.write_buffer(
                        buffer,
                        0,
                        bytemuck::cast_slice(&[*default_value]),
                    );
                } else {
                    panic!("No uniform value or default for {buffer_name}");
                }
            }
        }
    }

    name
}

pub fn create_user_pipeline(
    name: &str,
    pass_data: &MeshDrawData,
    shader: &Shader,
    context: &GraphicsContext,
    texture_layout: &Arc<wgpu::BindGroupLayout>,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    enable_z_buffer: bool,
) -> UserRenderPipeline {
    info!("Creating pipeline for shader: {:?}", shader.id);

    let mut layout_entries = Vec::new();
    let mut bind_group_entries = Vec::new();
    let mut buffers = HashMap::new();

    for (uniform_name, binding) in shader.bindings.iter() {
        let uniform_def = shader.uniform_defs.get(uniform_name).unwrap();

        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: *binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        let uniform_buffer_usage =
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;

        match uniform_def {
            UniformDef::F32(maybe_default) => {
                if let Some(value) = maybe_default {
                    let buffer = context.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some(&format!(
                                "User UB: {} (default={})",
                                uniform_name, value
                            )),
                            contents: bytemuck::cast_slice(&[*value]),
                            usage: uniform_buffer_usage,
                        },
                    );

                    buffers.insert(uniform_name.to_string(), buffer);
                } else {
                    let buffer =
                        context.device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some(&format!(
                                "User UB: {} (no-default)",
                                uniform_name
                            )),
                            size: std::mem::size_of::<f32>() as u64,
                            usage: uniform_buffer_usage,
                            mapped_at_creation: false,
                        });

                    buffers.insert(uniform_name.to_string(), buffer);
                }
            }
            UniformDef::Custom { .. } => {
                unimplemented!("custom uniforms aren't available yet");
            }
        };
    }

    for (name, binding) in shader.bindings.iter() {
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: *binding,
            resource: buffers.get(name).unwrap().as_entire_binding(),
        });
    }

    let user_layout = context.device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("User Layout: {}", name)),
            entries: &layout_entries,
        },
    );

    let pipeline = create_render_pipeline_with_layout(
        name,
        &context.device,
        wgpu::TextureFormat::Rgba16Float,
        &[&texture_layout, &camera_bind_group_layout, &user_layout],
        &[SpriteVertex::desc()],
        shader,
        pass_data.blend_mode,
        enable_z_buffer,
    )
    .unwrap();

    let bind_group =
        context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("User Bind Group"),
            layout: &user_layout,
            entries: &bind_group_entries,
        });

    UserRenderPipeline { pipeline, layout: user_layout, bind_group, buffers }
}
