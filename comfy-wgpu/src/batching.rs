use crate::*;

pub fn run_batched_render_passes(
    c: &mut WgpuRenderer,
    surface_view: &wgpu::TextureView,
    params: &DrawParams,
    sprite_shader_id: ShaderId,
    error_shader_id: ShaderId,
) {
    span_with_timing!("run_batched_render_passes");

    // TODO: ...
    // TODO: ...
    let _empty_pass = (
        MeshGroupKey {
            z_index: 0,
            blend_mode: BlendMode::Alpha,
            texture_id: TextureHandle::from_path("1px"),
            shader: None,
            render_target: None,
        },
        RenderPassData {
            z_index: 0,
            blend_mode: BlendMode::Alpha,
            texture: TextureHandle::from_path("1px"),
            shader: None,
            render_target: None,
            data: SmallVec::new(),
        },
    );

    let mut is_first = true;

    let queues = consume_render_queues();

    // let render_passes = {
    //     span_with_timing!("collect_render_passes");
    //
    //     let mut render_passes =
    //         HashMap::<MeshGroupKey, Vec<RenderPassData>>::new();
    //
    //     for (key, queue) in queues.into_iter() {
    //         render_passes.entry(key).or_default().push(RenderPassData {
    //             z_index: key.z_index,
    //             blend_mode: key.blend_mode,
    //             shader: key.shader,
    //             render_target: key.render_target,
    //             texture: key.texture_id,
    //             data: queue.into(),
    //         });
    //     }
    //
    //     render_passes
    // };

    for (key, mut meshes) in
        queues.into_iter().sorted_by_key(|(k, _)| k.z_index)
    {
        let _span = span!("blend/shader/target group");

        // TODO: add this back later
        if get_y_sort(key.z_index) {
            meshes.sort_by_key(|mesh| OrderedFloat::<f32>(-mesh.origin.y));
        }

        render_meshes(
            c,
            is_first,
            params.clear_color,
            MeshDrawData {
                blend_mode: key.blend_mode,
                texture: key.texture_id,
                shader: key.shader,
                render_target: key.render_target,
                data: meshes.into(),
            },
            surface_view,
            sprite_shader_id,
            error_shader_id,
        );

        perf_counter_inc("render passes", 1);

        is_first = false;
    }

    {
        span_with_timing!("prepare_particles");

        for (blend_mode, group) in
            &params.particle_queue.iter().group_by(|draw| draw.blend_mode)
        {
            for (tex_handle, group) in
                &group.into_iter().group_by(|draw| draw.texture)
            {
                for draw in group {
                    // particle_queue.push(RenderPassData {
                    //     // TODO: this is probably wrong
                    //     z_index: draw.position.z as i32,
                    //     blend_mode,
                    //     texture: tex_handle,
                    //     shader: None,
                    //     render_target: None,
                    //     data: DrawData::Particles(vec![*draw]),
                    // });

                    render_particles(
                        c,
                        is_first,
                        ParticleDrawData {
                            blend_mode,
                            texture: tex_handle,
                            data: vec![*draw],
                        },
                        params.clear_color,
                        surface_view,
                        sprite_shader_id,
                    );

                    perf_counter("particle draws", 1);
                    is_first = false;
                }
            }
        }
    }

    if is_first {
        render_meshes(
            c,
            is_first,
            params.clear_color,
            MeshDrawData {
                blend_mode: BlendMode::Alpha,
                texture: TextureHandle::from_path("1px"),
                shader: None,
                render_target: None,
                data: SmallVec::new(),
            },
            surface_view,
            sprite_shader_id,
            error_shader_id,
        );
    }
}

// TODO: Pass shader separately
pub fn render_meshes(
    c: &mut WgpuRenderer,
    is_first: bool,
    clear_color: Color,
    pass_data: MeshDrawData,
    surface_view: &wgpu::TextureView,
    sprite_shader_id: ShaderId,
    _error_shader_id: ShaderId,
) {
    let _span = span!("render_meshes");

    let pipeline_name = ensure_pipeline_exists(c, &pass_data, sprite_shader_id);

    perf_counter_inc("batch-count", 1);

    let tex_handle = pass_data.texture;
    let _span = span!("texture");

    let mut all_vertices: Vec<SpriteVertex> = vec![];
    let mut all_indices = vec![];

    for mesh in pass_data.data.into_iter() {
        let offset = all_vertices.len() as u32;
        all_vertices.extend(&mesh.vertices);
        all_indices.extend(mesh.indices.iter().map(|x| *x + offset));
    }

    c.vertex_buffer.ensure_size_and_copy(
        &c.context.device,
        &c.context.queue,
        bytemuck::cast_slice(all_vertices.as_slice()),
    );

    c.index_buffer.ensure_size_and_copy(
        &c.context.device,
        &c.context.queue,
        bytemuck::cast_slice(all_indices.as_slice()),
    );

    let textures = c.textures.lock();
    let render_targets = c.render_targets.borrow();

    let mut encoder = c.context.device.simple_encoder("Mesh Render Encoder");

    {
        let clear_color = if is_first { Some(clear_color) } else { None };

        let target_view = if let Some(render_target) = pass_data.render_target {
            &render_targets
                .get(&render_target)
                .expect("user render target must exist when used")
                .view
        } else if c.post_processing_effects.borrow().iter().any(|x| x.enabled) {
            &c.first_pass_texture.texture.view
        } else {
            surface_view
        };

        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mesh Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_to_clear_op(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: depth_stencil_attachment(
                    c.enable_z_buffer,
                    &c.depth_texture.view,
                    is_first,
                ),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        let mesh_pipeline = c
            .user_pipelines
            .get(&pipeline_name)
            .map(RenderPipeline::User)
            .or_else(|| {
                c.pipelines.get(&pipeline_name).map(RenderPipeline::Wgpu)
            })
            .expect("ensured pipeline must exist within the same frame");


        match &mesh_pipeline {
            RenderPipeline::User(pipeline) => {
                render_pass.set_pipeline(&pipeline.pipeline);
            }
            RenderPipeline::Wgpu(pipeline) => {
                render_pass.set_pipeline(pipeline);
            }
        }

        render_pass.set_vertex_buffer(0, c.vertex_buffer.buffer.slice(..));

        if !all_indices.is_empty() {
            render_pass.set_index_buffer(
                c.index_buffer.buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
        }

        let tex_bind_group = match tex_handle {
            TextureHandle::RenderTarget(render_target_id) => {
                &render_targets.get(&render_target_id).unwrap().bind_group
            }
            _ => {
                &textures
                    .get(&tex_handle)
                    .unwrap_or_else(|| {
                        textures
                            .get(&texture_id("error"))
                            .expect("error texture must exist")
                    })
                    .bind_group
            }
        };

        render_pass.set_bind_group(0, tex_bind_group, &[]);
        render_pass.set_bind_group(1, &c.camera_bind_group, &[]);

        match &mesh_pipeline {
            RenderPipeline::User(pipeline) => {
                render_pass.set_bind_group(2, &pipeline.bind_group, &[]);
            }
            RenderPipeline::Wgpu(_) => {}
        }

        if all_indices.is_empty() {
            render_pass.draw(0..all_vertices.len() as u32, 0..1);
        } else {
            render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);
        }
    }

    c.context.queue.submit(std::iter::once(encoder.finish()));
}

pub fn render_particles(
    c: &mut WgpuRenderer,
    is_first: bool,
    pass_data: ParticleDrawData,
    clear_color: Color,
    surface_view: &wgpu::TextureView,
    sprite_shader_id: ShaderId,
) {
    let _span = span!("render_particles");

    let target_view =
        if c.post_processing_effects.borrow().iter().any(|x| x.enabled) {
            &c.first_pass_texture.texture.view
        } else {
            surface_view
        };

    let textures = c.textures.lock();

    let particle_pipeline = {
        let name = format!(
            "Particle {:?} {:?}",
            pass_data.blend_mode, c.enable_z_buffer
        );

        c.pipelines.entry(name.clone()).or_insert_with(|| {
            create_render_pipeline_with_layout(
                &name,
                &c.context.device,
                // c.config.format,
                wgpu::TextureFormat::Rgba16Float,
                &[&c.texture_layout, &c.camera_bind_group_layout],
                &[SpriteVertex::desc()],
                &c.shaders.borrow().get(sprite_shader_id).unwrap().clone(),
                pass_data.blend_mode,
                c.enable_z_buffer,
            )
            .expect("particle pipeline creation failed")
        })
    };

    let mut all_vertices: Vec<SpriteVertex> = vec![];
    let mut all_indices: Vec<u32> = vec![];

    for draw in pass_data.data {
        let size = draw.size;

        let tex_size = ASSETS
            .borrow()
            .texture_image_map
            .lock()
            .get(&pass_data.texture)
            .map(|image| vec2(image.width() as f32, image.height() as f32))
            .unwrap_or(Vec2::ONE);

        let tex_width = tex_size.x;
        let tex_height = tex_size.y;

        let vertices = rotated_rectangle(
            // TODO: fix particle Z
            draw.position,
            RawDrawParams {
                dest_size: Some(size),
                rotation: draw.rotation,
                source_rect: draw.source_rect,
                ..Default::default()
            },
            tex_width,
            tex_height,
            draw.color,
            // TODO: scrolling particle offset?
            Vec2::ZERO,
        );

        let len = all_vertices.len() as u32;
        all_indices.extend_from_slice(&[
            len,
            2 + len,
            1 + len,
            len,
            3 + len,
            2 + len,
        ]);

        all_vertices.extend(vertices);
    }

    let mut encoder =
        c.context.device.simple_encoder("Particle Render Encoder");

    c.vertex_buffer.ensure_size_and_copy(
        &c.context.device,
        &c.context.queue,
        bytemuck::cast_slice(all_vertices.as_slice()),
    );

    c.index_buffer.ensure_size_and_copy(
        &c.context.device,
        &c.context.queue,
        bytemuck::cast_slice(all_indices.as_slice()),
    );

    {
        let clear_color = if is_first { Some(clear_color) } else { None };

        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Particle Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_to_clear_op(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                // depth_stencil_attachment: Some(
                //     wgpu::RenderPassDepthStencilAttachment {
                //         view: &c.depth_texture.view,
                //         depth_ops: Some(wgpu::Operations {
                //             load: clear_depth,
                //             store: true,
                //         }),
                //         stencil_ops: None,
                //     },
                // ),
                depth_stencil_attachment: depth_stencil_attachment(
                    c.enable_z_buffer,
                    &c.depth_texture.view,
                    is_first,
                ),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        render_pass.set_pipeline(particle_pipeline);
        render_pass.set_vertex_buffer(0, c.vertex_buffer.buffer.slice(..));

        if !all_indices.is_empty() {
            render_pass.set_index_buffer(
                c.index_buffer.buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
        }

        let tex_bind_group = &textures
            .get(&pass_data.texture)
            .unwrap_or_else(|| textures.get(&texture_id("error")).unwrap())
            .bind_group;

        render_pass.set_bind_group(0, tex_bind_group, &[]);
        render_pass.set_bind_group(1, &c.camera_bind_group, &[]);

        if all_indices.is_empty() {
            render_pass.draw(0..all_vertices.len() as u32, 0..1);
        } else {
            render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);
        }
    }

    c.context.queue.submit(std::iter::once(encoder.finish()));
}
