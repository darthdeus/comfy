use std::sync::atomic::AtomicU64;

use crate::*;

static GENERATED_RENDER_TARGET_IDS: AtomicU64 = AtomicU64::new(0);
static SHADER_IDS: AtomicU64 = AtomicU64::new(0);

/// Generates a new ShaderId. This is intended for internal use only.
pub fn gen_shader_id() -> ShaderId {
    let id = SHADER_IDS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    info!("Generated ShaderId: {}", id);

    ShaderId(id)
}

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

#[derive(Clone, Debug)]
pub enum ShaderError {
    CompileError(String),
}

pub fn create_shader(
    shaders: &mut ShaderMap,
    name: &str,
    source: &str,
    _uniforms_with_defaults: HashMap<&'static str, UniformDesc>,
) -> Result<ShaderId, ShaderError> {
    let id = gen_shader_id();

    if shaders.contains_key(&id) {
        return Err(ShaderError::CompileError(format!(
            "Shader with name '{}' already exists",
            name
        )));
    }

    shaders.insert(id, Shader {
        id,
        name: format!("{} Shader", name),
        source: source.to_string(),
    });

    Ok(id)
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

pub fn set_uniform(_name: &str, _value: Uniform) {}
