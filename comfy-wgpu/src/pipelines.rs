use crate::*;

pub fn get_or_create_pipeline<'a>(
    enable_z_buffer: bool,
    user_pipelines: &'a mut UserPipelineMap,
    context: &'a GraphicsContext,
    texture_layout: &Arc<wgpu::BindGroupLayout>,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    pass_data: &MeshDrawData,
    sprite_shader_id: ShaderId,
    maybe_shader: Option<&Shader>,
    maybe_shader_instance: &Option<ShaderInstance>,
    pipelines: &'a mut PipelineMap,
    shaders: &'a ShaderMap,
) -> RenderPipeline<'a> {
    let name = format!(
        "{} {:?} {:?} {:?}",
        if maybe_shader_instance.is_some() {
            "USER(Mesh)"
        } else {
            "BUILTIN(Mesh)"
        },
        pass_data.blend_mode,
        maybe_shader,
        enable_z_buffer
    );

    let mesh_pipeline = if let Some(shader) = maybe_shader {
        RenderPipeline::User(user_pipelines.entry(name.clone()).or_insert_with(
            || {
                info!("Creating pipeline for shader: {:?}", shader.id);

                let mut layout_entries = Vec::new();
                let mut bind_group_entries = Vec::new();
                let mut buffers = HashMap::new();

                for (uniform_name, binding) in shader.bindings.iter() {
                    let uniform_def =
                        shader.uniform_defs.get(uniform_name).unwrap();

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

                    let uniform_buffer_usage = wgpu::BufferUsages::UNIFORM |
                        wgpu::BufferUsages::COPY_DST;

                    match uniform_def {
                        UniformDef::F32(maybe_default) => {
                            if let Some(value) = maybe_default {
                                let buffer = context.device.create_buffer_init(
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some(&format!(
                                            "User UB: {} (default={})",
                                            uniform_name, value
                                        )),
                                        contents: bytemuck::cast_slice(&[
                                            *value,
                                        ]),
                                        usage: uniform_buffer_usage,
                                    },
                                );

                                buffers
                                    .insert(uniform_name.to_string(), buffer);
                            } else {
                                let buffer = context.device.create_buffer(
                                    &wgpu::BufferDescriptor {
                                        label: Some(&format!(
                                            "User UB: {} (no-default)",
                                            uniform_name
                                        )),
                                        size: std::mem::size_of::<f32>() as u64,
                                        usage: uniform_buffer_usage,
                                        mapped_at_creation: false,
                                    },
                                );

                                buffers
                                    .insert(uniform_name.to_string(), buffer);
                            }
                        }
                        UniformDef::Custom { .. } => {
                            unimplemented!(
                                "custom uniforms aren't available yet"
                            );
                        }
                    };
                }

                for (name, binding) in shader.bindings.iter() {
                    bind_group_entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: buffers
                            .get(name)
                            .unwrap()
                            .as_entire_binding(),
                    });
                }

                let user_layout = context.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: Some(&format!("User Layout: {}", name)),
                        entries: &layout_entries,
                    },
                );

                let pipeline = create_render_pipeline_with_layout(
                    &name,
                    &context.device,
                    wgpu::TextureFormat::Rgba16Float,
                    &[texture_layout, camera_bind_group_layout, &user_layout],
                    &[SpriteVertex::desc()],
                    shader,
                    pass_data.blend_mode,
                    enable_z_buffer,
                )
                .unwrap();

                let bind_group = context.device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        label: Some("User Bind Group"),
                        layout: &user_layout,
                        entries: &bind_group_entries,
                    },
                );

                UserRenderPipeline {
                    pipeline,
                    layout: user_layout,
                    bind_group,
                    buffers,
                }
            },
        ))
    } else {
        RenderPipeline::Wgpu(pipelines.entry(name.clone()).or_insert_with(
            || {
                create_render_pipeline_with_layout(
                    &name,
                    &context.device,
                    wgpu::TextureFormat::Rgba16Float,
                    &[texture_layout, camera_bind_group_layout],
                    &[SpriteVertex::desc()],
                    shaders.get(sprite_shader_id).unwrap(),
                    pass_data.blend_mode,
                    enable_z_buffer,
                )
                .unwrap()
            },
        ))
    };

    if let RenderPipeline::User(ref user_pipeline) = mesh_pipeline {
        if let Some(shader_instance) = maybe_shader_instance {
            let shader = shaders.get(shader_instance.id).unwrap();

            for (buffer_name, buffer) in
                user_pipeline.buffers.iter().sorted_by_key(|x| x.0)
            {
                if let Some(Uniform::F32(OrderedFloat(value))) =
                    shader_instance.uniforms.get(buffer_name)
                {
                    context.queue.write_buffer(
                        buffer,
                        0,
                        bytemuck::cast_slice(&[*value]),
                    );
                } else if let UniformDef::F32(Some(default_value)) =
                    shader.uniform_defs.get(buffer_name).unwrap()
                {
                    context.queue.write_buffer(
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

    mesh_pipeline
}
