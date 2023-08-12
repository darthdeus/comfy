use crate::*;

pub struct MeshDrawData {
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub data: smallvec::SmallVec<[MeshDraw; 1]>,
}

pub struct ParticleDrawData {
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub data: Vec<ParticleDraw>,
}

pub struct RenderPassData {
    pub z_index: i32,
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub data: DrawData,
    // Meshes {
    //     meshes: Vec<MeshDraw>,
    // },
    //
    // Particles {
    //     z_index: i32,
    //     blend_mode: BlendMode,
    //     draw_mode: DrawMode,
    //     texture: TextureHandle,
    //     particles: Vec<ParticleDraw>,
    // },
}

// TODO: enum has a large difference between member sizes
#[allow(clippy::large_enum_variant)]
pub enum DrawData {
    Meshes(smallvec::SmallVec<[MeshDraw; 1]>),
    Particles(Vec<ParticleDraw>),
}

pub fn collect_render_passes(params: &DrawParams) -> Vec<RenderPassData> {
    span_with_timing!("collect_render_passes");

    let white_px = TextureHandle::from_path("1px");
    let mut result = vec![];

    {
        span_with_timing!("prepare meshes");

        // Meshes
        for (blend_mode, group) in &params
            .mesh_queue
            .iter()
            .group_by(|draw| (draw.texture_params.blend_mode))
        {
            let _span = span!("blend_mode");

            for ((tex_handle, _tex_params), group) in &group
                .into_iter()
                .group_by(|draw| (draw.mesh.texture, &draw.texture_params))
            {
                perf_counter_inc("batch-count", 1);

                let tex_handle = tex_handle.unwrap_or(white_px);

                let _span = span!("texture");


                // TODO: no need to sort anymore
                for draw in group.sorted_by_key(|draw| draw.mesh.z_index) {
                    result.push(RenderPassData {
                        z_index: draw.mesh.z_index,
                        blend_mode,
                        texture: tex_handle,
                        data: DrawData::Meshes([draw.clone()].into()),
                    });
                }
            }
        }
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
                    result.push(RenderPassData {
                        // TODO: this is probably wrong
                        z_index: draw.position.z as i32,
                        blend_mode,
                        texture: tex_handle,
                        data: DrawData::Particles(vec![*draw]),
                    });
                }
            }
        }
    }

    result
}

// pub fn render_meshes(
//     &mut self,
//     clear_color: Color,
//     mesh_queue: &mut Vec<MeshDraw>,
//     surface_view: &wgpu::TextureView,
// ) {
//     let _span = span!("render_meshes");
//
//     let target_view = if self.post_processing_effects.iter().any(|x| x.enabled)
//     {
//         &self.first_pass_texture.view
//     } else {
//         &surface_view
//     };
//
//     let textures = self.textures.lock();
//
//     let mut is_first = true;
//     let white_px = TextureHandle::from_path("1px");
//
//     for ((blend_mode, draw_mode), group) in &mesh_queue
//         .iter()
//         .group_by(|draw| (draw.texture_params.blend_mode, draw.mesh.draw_mode))
//     {
//         let _span = span!("blend_mode");
//
//         let mesh_pipeline = {
//             let name = format!("Mesh {:?} {:?}", blend_mode, draw_mode);
//
//             self.pipelines.entry(name.clone().into()).or_insert_with(|| {
//                 create_render_pipeline_with_layout(
//                     &name,
//                     &self.context.device,
//                     self.config.format,
//                     &[
//                         &self.texture_bind_group_layout,
//                         &self.camera_bind_group_layout,
//                         &self.lights_bind_group_layout,
//                         &self.global_lighting_params_bind_group_layout,
//                     ],
//                     &[SpriteVertex::desc()],
//                     reloadable_wgsl_shader!("sprite"),
//                     blend_mode,
//                     draw_mode,
//                 )
//             })
//         };
//
//         for ((tex_handle, _tex_params), group) in &group
//             .into_iter()
//             .group_by(|draw| (draw.mesh.texture, &draw.texture_params))
//         {
//             perf_counter_inc("batch-count", 1);
//
//             let tex_handle = tex_handle.unwrap_or(white_px);
//             let _span = span!("texture");
//             // let tex_handle = TextureHandle::from_path("1px");
//
//
//             let mut all_vertices: Vec<SpriteVertex> = vec![];
//             let mut all_indices = vec![];
//
//             // let shader = if tex_params.shader.is_some() {
//             //     &self.shaders[0]
//             // } else {
//             //     &self.batch.shader
//             // };
//             //
//             // set_uniforms(shader);
//             // shader.use_shader();
//
//             for draw in group.sorted_by_key(|draw| draw.mesh.z_index) {
//                 // for draw in group {
//                 // let mut mesh = draw.mesh.clone();
//
//                 // all_indices.extend(
//                 //     mesh.indices
//                 //         .iter()
//                 //         .cloned()
//                 //         .map(|x| x as u32 + all_vertices.len() as u32),
//                 // );
//                 //
//                 // all_vertices.extend(mesh.vertices.drain(..));
//
//                 let offset = all_vertices.len() as u32;
//                 all_vertices.extend(&draw.mesh.vertices);
//                 all_indices.extend(
//                     draw.mesh.indices.iter().map(|x| *x as u32 + offset),
//                 );
//             }
//
//             let mut encoder = self.device.simple_encoder("Mesh Render Encoder");
//
//             self.vertex_buffer.ensure_size_and_copy(
//                 &self.device,
//                 &self.queue,
//                 bytemuck::cast_slice(all_vertices.as_slice()),
//             );
//
//             self.index_buffer.ensure_size_and_copy(
//                 &self.device,
//                 &self.queue,
//                 bytemuck::cast_slice(all_indices.as_slice()),
//             );
//
//             {
//                 let (clear_color, clear_depth) = if is_first {
//                     is_first = false;
//                     (Some(clear_color), wgpu::LoadOp::Clear(1.0))
//                 } else {
//                     (None, wgpu::LoadOp::Load)
//                 };
//
//                 // let mut render_pass = encoder.simple_render_pass(
//                 //     "Mesh Render Pass",
//                 //     clear_color,
//                 //     target_view,
//                 //     Some(wgpu::RenderPassDepthStencilAttachment {
//                 //         view: depth_texture_view,
//                 //         depth_ops: Some(wgpu::Operations {
//                 //             load: wgpu::LoadOp::Clear(1.0),
//                 //             store: true,
//                 //         }),
//                 //         stencil_ops: None,
//                 //     }),
//                 // );
//
//                 let mut render_pass =
//                     encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                         label: Some("Mesh Render Pass"),
//                         color_attachments: &[Some(
//                             wgpu::RenderPassColorAttachment {
//                                 view: target_view,
//                                 resolve_target: None,
//                                 ops: wgpu::Operations {
//                                     load: color_to_clear_op(clear_color),
//                                     store: true,
//                                 },
//                             },
//                         )],
//                         depth_stencil_attachment: Some(
//                             wgpu::RenderPassDepthStencilAttachment {
//                                 view: &self.depth_texture.view,
//                                 depth_ops: Some(wgpu::Operations {
//                                     load: clear_depth,
//                                     store: true,
//                                 }),
//                                 stencil_ops: None,
//                             },
//                         ),
//                     });
//
//                 render_pass.set_pipeline(&mesh_pipeline);
//                 render_pass
//                     .set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
//
//                 if all_indices.len() > 0 {
//                     render_pass.set_index_buffer(
//                         self.index_buffer.buffer.slice(..),
//                         wgpu::IndexFormat::Uint32,
//                     );
//                 }
//
//                 let tex_bind_group = &textures
//                     .get(&tex_handle)
//                     .unwrap_or_else(|| {
//                         &textures.get(&texture_id("error")).unwrap()
//                     })
//                     .0;
//
//                 render_pass.set_bind_group(0, tex_bind_group, &[]);
//                 render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
//                 render_pass.set_bind_group(2, &self.lights_bind_group, &[]);
//                 render_pass.set_bind_group(
//                     3,
//                     &self.global_lighting_params_bind_group,
//                     &[],
//                 );
//
//                 if all_indices.len() == 0 {
//                     render_pass.draw(0..all_vertices.len() as u32, 0..1);
//                 } else {
//                     render_pass.draw_indexed(
//                         0..all_indices.len() as u32,
//                         0,
//                         0..1,
//                     );
//                 }
//             }
//
//             self.queue.submit(std::iter::once(encoder.finish()));
//         }
//     }
// }

// pub fn render_particles(
//     &mut self,
//     particle_queue: &mut Vec<ParticleDraw>,
//     surface_view: &wgpu::TextureView,
// ) {
//     let _span = span!("render_particles");
//
//     let target_view = if self.post_processing_effects.iter().any(|x| x.enabled)
//     {
//         &self.first_pass_texture.view
//     } else {
//         &surface_view
//     };
//
//     let textures = self.textures.lock();
//
//     for (blend_mode, group) in
//         &particle_queue.iter().group_by(|draw| draw.blend_mode)
//     {
//         let particle_pipeline = {
//             let name = format!("Particle {:?}", blend_mode);
//
//             self.pipelines.entry(name.clone().into()).or_insert_with(|| {
//                 create_render_pipeline_with_layout(
//                     &name,
//                     &self.context.device,
//                     self.config.format,
//                     &[
//                         &self.texture_bind_group_layout,
//                         &self.camera_bind_group_layout,
//                         &self.lights_bind_group_layout,
//                         &self.global_lighting_params_bind_group_layout,
//                     ],
//                     &[SpriteVertex::desc()],
//                     reloadable_wgsl_shader!("sprite"),
//                     blend_mode,
//                     DrawMode::TriangleList,
//                 )
//             })
//         };
//
//         for (tex_handle, group) in
//             &group.into_iter().group_by(|draw| draw.texture)
//         {
//             let mut all_vertices: Vec<SpriteVertex> = vec![];
//             let mut all_indices: Vec<u32> = vec![];
//
//             for draw in group {
//                 let size = draw.size;
//
//                 let tex_size = ASSETS
//                     .borrow()
//                     .texture_image_map
//                     .lock()
//                     .get(&tex_handle)
//                     .map(|image| {
//                         vec2(image.width() as f32, image.height() as f32)
//                     })
//                     .unwrap_or(Vec2::ONE);
//
//                 let tex_width = tex_size.x;
//                 let tex_height = tex_size.y;
//
//                 let vertices = rotated_rectangle(
//                     // TODO: fix particle Z
//                     draw.position,
//                     RawDrawParams {
//                         dest_size: Some(size),
//                         rotation: draw.rotation,
//                         source_rect: draw.source_rect,
//                         ..Default::default()
//                     },
//                     tex_width,
//                     tex_height,
//                     draw.color,
//                 );
//
//                 let len = all_vertices.len() as u32;
//                 all_indices.extend_from_slice(&[
//                     0 + len,
//                     2 + len,
//                     1 + len,
//                     0 + len,
//                     3 + len,
//                     2 + len,
//                 ]);
//
//                 all_vertices.extend(vertices);
//             }
//
//             let mut encoder =
//                 self.device.simple_encoder("Particle Render Encoder");
//
//             self.vertex_buffer.ensure_size_and_copy(
//                 &self.device,
//                 &self.queue,
//                 bytemuck::cast_slice(all_vertices.as_slice()),
//             );
//
//             self.index_buffer.ensure_size_and_copy(
//                 &self.device,
//                 &self.queue,
//                 bytemuck::cast_slice(all_indices.as_slice()),
//             );
//
//             {
//                 let clear_color = None;
//
//                 let mut render_pass =
//                     encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                         label: Some("Mesh Render Pass"),
//                         color_attachments: &[Some(
//                             wgpu::RenderPassColorAttachment {
//                                 view: target_view,
//                                 resolve_target: None,
//                                 ops: wgpu::Operations {
//                                     load: color_to_clear_op(clear_color),
//                                     store: true,
//                                 },
//                             },
//                         )],
//                         depth_stencil_attachment: Some(
//                             wgpu::RenderPassDepthStencilAttachment {
//                                 view: &self.depth_texture.view,
//                                 depth_ops: Some(wgpu::Operations {
//                                     load: wgpu::LoadOp::Load,
//                                     store: true,
//                                 }),
//                                 stencil_ops: None,
//                             },
//                         ),
//                     });
//
//                 render_pass.set_pipeline(&particle_pipeline);
//                 render_pass
//                     .set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
//
//                 if all_indices.len() > 0 {
//                     render_pass.set_index_buffer(
//                         self.index_buffer.buffer.slice(..),
//                         wgpu::IndexFormat::Uint32,
//                     );
//                 }
//
//                 let tex_bind_group = &textures
//                     .get(&tex_handle)
//                     .unwrap_or_else(|| {
//                         &textures.get(&texture_id("error")).unwrap()
//                     })
//                     .0;
//
//                 render_pass.set_bind_group(0, tex_bind_group, &[]);
//                 render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
//                 render_pass.set_bind_group(2, &self.lights_bind_group, &[]);
//                 render_pass.set_bind_group(
//                     3,
//                     &self.global_lighting_params_bind_group,
//                     &[],
//                 );
//
//                 if all_indices.len() == 0 {
//                     render_pass.draw(0..all_vertices.len() as u32, 0..1);
//                 } else {
//                     render_pass.draw_indexed(
//                         0..all_indices.len() as u32,
//                         0,
//                         0..1,
//                     );
//                 }
//             }
//
//             self.queue.submit(std::iter::once(encoder.finish()));
//         }
//     }
// }
