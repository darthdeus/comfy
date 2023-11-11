use std::sync::atomic::AtomicU64;

use crate::*;

static GENERATED_RENDER_TARGET_IDS: AtomicU64 = AtomicU64::new(0);

/// Allocates a new render target for later use. If a label is provided
/// it'll be used to set the debug name so graphic debuggers like RenderDoc
/// can display it properly.
pub fn gen_render_target(_label: Option<&str>) -> RenderTargetId {
    // TODO: use the label
    //
    let id = GENERATED_RENDER_TARGET_IDS
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    RenderTargetId::Generated(id)
}

pub fn ensure_pipeline_exists(
    c: &mut WgpuRenderer,
    pass_data: &MeshDrawData,
    sprite_shader_id: ShaderId,
) -> String {
    let shaders = c.shaders.borrow();

    let maybe_shader_instance =
        pass_data.data.first().and_then(|x| x.shader.clone());

    let maybe_shader = maybe_shader_instance
        .as_ref()
        .and_then(|instance| shaders.get(instance.id));

    let name = format!(
        "{} {:?} {:?} {:?}",
        if maybe_shader_instance.is_some() {
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
        if let Some(shader_instance) = maybe_shader_instance {
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
