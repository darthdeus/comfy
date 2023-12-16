use crate::*;

pub struct MeshDrawData {
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub shader: ShaderInstanceId,
    pub render_target: RenderTargetId,
    pub data: Vec<Mesh>,
}

pub struct ParticleDrawData {
    pub blend_mode: BlendMode,
    pub texture: TextureHandle,
    pub data: Vec<ParticleDraw>,
}
