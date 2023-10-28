use crate::*;

#[derive(Clone, Debug)]
pub enum RenderTargetId {
    Named(String),
    Generated(u64),
}

#[derive(Copy, Clone, Debug)]
pub struct ShaderId(u64);

#[derive(Clone, Debug)]
pub enum ShaderError {
    CompileError(String),
}

pub fn create_shader(
    _source: &str,
    _uniforms_with_defaults: HashMap<&'static str, UniformDesc>,
) -> Result<ShaderId, ShaderError> {
    Ok(ShaderId(10))
}

#[derive(Clone, Debug)]
pub enum UniformDesc {
    F32(f32),
    Custom { default_data: Vec<u8>, wgsl_decl: String },
}


#[derive(Clone, Debug)]
pub enum Uniform {
    F32(f32),
    Custom(Vec<u8>),
}

pub fn set_shader(_shader_id: ShaderId) {}

pub fn set_uniform(_name: &str, _value: Uniform) {}
