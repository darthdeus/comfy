use crate::*;

pub struct MeshDrawData {
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub shader: Option<ShaderInstance>,
    pub render_target: Option<RenderTargetId>,
    pub data: smallvec::SmallVec<[MeshDraw; 1]>,
}

pub struct ParticleDrawData {
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub data: Vec<ParticleDraw>,
}

#[derive(Debug)]
pub struct RenderPassData {
    pub z_index: i32,
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub data: DrawData,
    pub shader: Option<ShaderInstance>,
    pub render_target: Option<RenderTargetId>,
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
#[derive(Debug)]
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
        for ((blend_mode, shader, render_target), group) in
            &params.mesh_queue.iter().group_by(|draw| {
                (
                    draw.texture_params.blend_mode,
                    &draw.shader,
                    draw.render_target,
                )
            })
        {
            let _span = span!("blend_mode");

            for ((tex_handle, _tex_params), group) in &group
                .into_iter()
                .group_by(|draw| (draw.mesh.texture, &draw.texture_params))
            {
                perf_counter_inc("batch-count", 1);

                let tex_handle = tex_handle.unwrap_or(white_px);

                let _span = span!("texture");

                // TODO: do we still need to sort here?
                let mut sorted_by_z =
                    group.sorted_by_key(|draw| draw.mesh.z_index).collect_vec();

                if !sorted_by_z.is_empty() &&
                    get_y_sort(sorted_by_z[0].mesh.z_index)
                {
                    sorted_by_z.sort_by_cached_key(|draw| {
                        OrderedFloat::<f32>(
                            // TODO: This is completely disgusting, but it does work ...
                            -draw
                                .mesh
                                .vertices
                                .iter()
                                .map(|v| v.position[1])
                                .sum::<f32>() /
                                draw.mesh.vertices.len() as f32,
                        )
                    });
                }

                for draw in sorted_by_z {
                    result.push(RenderPassData {
                        z_index: draw.mesh.z_index,
                        blend_mode,
                        shader: shader.clone(),
                        render_target,
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
                        shader: None,
                        render_target: None,
                        data: DrawData::Particles(vec![*draw]),
                    });
                }
            }
        }
    }

    if result.is_empty() {
        vec![RenderPassData {
            z_index: 0,
            blend_mode: BlendMode::Alpha,
            texture: white_px,
            shader: None,
            render_target: None,
            data: DrawData::Meshes(SmallVec::new()),
        }]
    } else {
        result
    }
}
