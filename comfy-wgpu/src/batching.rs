use crate::*;

pub fn draw_batched_render_passes(
    c: &mut WgpuRenderer,
    surface_view: &wgpu::TextureView,
    params: &DrawParams,
) {
    let render_passes = collect_render_passes(params);

    perf_counter("render pass blocks", render_passes.len() as u64);
    perf_counter(
        "mesh draws",
        render_passes
            .iter()
            .filter(|x| matches!(x.data, DrawData::Meshes(_)))
            .count() as u64,
    );

    perf_counter(
        "particle draws",
        render_passes
            .iter()
            .filter(|x| matches!(x.data, DrawData::Particles(_)))
            .count() as u64,
    );

    let mut is_first = true;

    let grouped_render_passes = render_passes
        .into_iter()
        .sorted_by_key(|p| p.z_index)
        .group_by(|p| p.z_index);

    for (_, z_index_group) in &grouped_render_passes {
        for ((blend_mode, shader), blend_group) in &z_index_group
            .sorted_by_key(|x| x.blend_mode)
            .group_by(|x| (x.blend_mode, x.shader.clone()))
        {
            let (meshes, particles) = blend_group.into_iter().fold(
                (vec![], vec![]),
                |mut acc, pass_data| {
                    match pass_data.data {
                        DrawData::Meshes(mesh_draw) => {
                            acc.0.push(MeshDrawData {
                                blend_mode,
                                shader: shader.clone(),
                                texture: pass_data.texture,
                                data: mesh_draw,
                            })
                        }
                        DrawData::Particles(particle_draw) => {
                            acc.1.push(ParticleDrawData {
                                blend_mode,
                                texture: pass_data.texture,
                                data: particle_draw,
                            })
                        }
                    }

                    acc
                },
            );

            for ((blend_mode, texture, shader), mesh_group) in &meshes
                .into_iter()
                .sorted_by_key(|x| x.texture)
                .group_by(|x| (x.blend_mode, x.texture, x.shader.clone()))
            {
                render_meshes(
                    c,
                    is_first,
                    params.clear_color,
                    MeshDrawData {
                        blend_mode,
                        texture,
                        shader: shader.clone(),
                        data: mesh_group
                            .flat_map(|x| x.data)
                            .collect_vec()
                            .into(),
                    },
                    surface_view,
                );

                perf_counter_inc("real_mesh_draw", 1);
                is_first = false;
            }

            for ((blend_mode, texture), particle_group) in
                &particles.into_iter().group_by(|x| (x.blend_mode, x.texture))
            {
                render_particles(
                    c,
                    is_first,
                    ParticleDrawData {
                        blend_mode,
                        texture,
                        data: particle_group.flat_map(|x| x.data).collect_vec(),
                    },
                    params.clear_color,
                    surface_view,
                );

                perf_counter_inc("real_particle_draw", 1);
                is_first = false;
            }

            is_first = false;
        }
    }
}

// TODO: Pass shader separately
pub fn render_meshes(
    c: &mut WgpuRenderer,
    is_first: bool,
    clear_color: Color,
    pass_data: MeshDrawData,
    surface_view: &wgpu::TextureView,
) {
    let _span = span!("render_meshes");

    let target_view =
        if c.post_processing_effects.borrow().iter().any(|x| x.enabled) {
            &c.first_pass_texture.view
        } else {
            surface_view
        };

    let textures = c.textures.lock();

    let _span = span!("blend_mode");

    let mesh_pipeline = {
        let shader = pass_data.data.get(0).and_then(|x| x.shader.clone());

        let name = format!(
            "Mesh {:?} {:?} {:?}",
            pass_data.blend_mode, shader, c.enable_z_buffer
        );

        c.pipelines.entry(name.clone()).or_insert_with(|| {
            info!("shader: {:?}", shader);

            let shader = match shader {
                Some(shader_instance) => {
                    c.shaders.borrow().get(&shader_instance.id).unwrap().clone()
                }
                None => reloadable_wgsl_shader!("sprite"),
            };

            create_render_pipeline_with_layout(
                &name,
                &c.context.device,
                wgpu::TextureFormat::Rgba16Float,
                &[
                    &c.texture_layout,
                    &c.camera_bind_group_layout,
                    &c.global_lighting_params_bind_group_layout,
                ],
                &[SpriteVertex::desc()],
                // reloadable_wgsl_shader!("sprite"),
                &shader,
                pass_data.blend_mode,
                c.enable_z_buffer,
            )
        })
    };

    perf_counter_inc("batch-count", 1);

    let tex_handle = pass_data.texture;
    let _span = span!("texture");
    // let tex_handle = TextureHandle::from_path("1px");

    let mut all_vertices: Vec<SpriteVertex> = vec![];
    let mut all_indices = vec![];

    for draw in pass_data.data.into_iter() {
        // for draw in group {
        // let mut mesh = draw.mesh.clone();

        // all_indices.extend(
        //     mesh.indices
        //         .iter()
        //         .cloned()
        //         .map(|x| x as u32 + all_vertices.len() as u32),
        // );
        //
        // all_vertices.extend(mesh.vertices.drain(..));

        let offset = all_vertices.len() as u32;
        all_vertices.extend(&draw.mesh.vertices);
        all_indices.extend(draw.mesh.indices.iter().map(|x| *x + offset));
    }

    // let all_vertices = mesh_draw[0].mesh.vertices.clone();
    // let all_indices = mesh_draw[0].mesh.indices.clone();

    let mut encoder = c.context.device.simple_encoder("Mesh Render Encoder");

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
                label: Some("Mesh Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_to_clear_op(clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: depth_stencil_attachment(
                    c.enable_z_buffer,
                    &c.depth_texture.view,
                    is_first,
                ),
            });

        render_pass.set_pipeline(mesh_pipeline);
        render_pass.set_vertex_buffer(0, c.vertex_buffer.buffer.slice(..));

        if !all_indices.is_empty() {
            render_pass.set_index_buffer(
                c.index_buffer.buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
        }

        let tex_bind_group = &textures
            .get(&tex_handle)
            .unwrap_or_else(|| textures.get(&texture_id("error")).unwrap())
            .0;

        render_pass.set_bind_group(0, tex_bind_group, &[]);
        render_pass.set_bind_group(1, &c.camera_bind_group, &[]);
        render_pass
            .set_bind_group(2, &c.global_lighting_params_bind_group, &[]);

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
) {
    let _span = span!("render_particles");

    let target_view =
        if c.post_processing_effects.borrow().iter().any(|x| x.enabled) {
            &c.first_pass_texture.view
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
                &[
                    &c.texture_layout,
                    &c.camera_bind_group_layout,
                    &c.global_lighting_params_bind_group_layout,
                ],
                &[SpriteVertex::desc()],
                // TODO: shaders.get_or_err(...)
                &reloadable_wgsl_shader!("sprite"),
                pass_data.blend_mode,
                c.enable_z_buffer,
            )
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
                        store: true,
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
            .0;

        render_pass.set_bind_group(0, tex_bind_group, &[]);
        render_pass.set_bind_group(1, &c.camera_bind_group, &[]);
        render_pass
            .set_bind_group(2, &c.global_lighting_params_bind_group, &[]);

        if all_indices.is_empty() {
            render_pass.draw(0..all_vertices.len() as u32, 0..1);
        } else {
            render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);
        }
    }

    c.context.queue.submit(std::iter::once(encoder.finish()));
}
