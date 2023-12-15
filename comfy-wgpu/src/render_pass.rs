use crate::*;

pub struct MeshDrawData {
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub shader: Option<ShaderInstanceId>,
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
    pub shader: Option<ShaderInstanceId>,
    pub render_target: Option<RenderTargetId>,
}

// TODO: enum has a large difference between member sizes
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum DrawData {
    Meshes(smallvec::SmallVec<[MeshDraw; 1]>),
    Particles(Vec<ParticleDraw>),
}

pub fn collect_render_passes(
    params: &DrawParams,
) -> HashMap<i32, Vec<RenderPassData>> {
    span_with_timing!("collect_render_passes");

    let mut result = vec![];
    let queues = consume_render_queues();

    perf_counter_inc("batch-count", queues.len() as u64);

    for (key, queue) in queues.into_iter() {
        let _span = span!("mesh group");

        let mut sorted_by_z = queue.meshes;

        if get_y_sort(key.z_index) {
            sorted_by_z
                .sort_by_key(|draw| OrderedFloat::<f32>(-draw.mesh.origin.y));
        }

        for draw in sorted_by_z {
            result.push(RenderPassData {
                z_index: draw.mesh.z_index,
                blend_mode: key.blend_mode,
                shader: key.shader,
                render_target: key.render_target,
                texture: key.texture_id,
                data: DrawData::Meshes([draw].into()),
            });
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

    let results = if result.is_empty() {
        vec![RenderPassData {
            z_index: 0,
            blend_mode: BlendMode::Alpha,
            texture: TextureHandle::from_path("1px"),
            shader: None,
            render_target: None,
            data: DrawData::Meshes(SmallVec::new()),
        }]
    } else {
        result
    };

    let mut map = HashMap::<i32, Vec<RenderPassData>>::new();

    for pass in results.into_iter() {
        map.entry(pass.z_index).or_default().push(pass);
    }

    map
}
