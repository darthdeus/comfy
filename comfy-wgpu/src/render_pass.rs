use crate::*;

pub struct MeshDrawData {
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub shader: ShaderInstanceId,
    pub render_target: RenderTargetId,
    pub data: smallvec::SmallVec<[Mesh; 1]>,
}

pub struct ParticleDrawData {
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub data: Vec<ParticleDraw>,
}

#[derive(Clone, Debug)]
pub struct RenderPassData {
    pub z_index: i32,
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub data: smallvec::SmallVec<[Mesh; 1]>,
    pub shader: Option<ShaderInstanceId>,
    pub render_target: Option<RenderTargetId>,
}
