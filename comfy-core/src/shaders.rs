use crate::*;
use std::sync::atomic::AtomicU64;

static SHADER_IDS: AtomicU64 = AtomicU64::new(0);
static GENERATED_RENDER_TARGET_IDS: AtomicU64 = AtomicU64::new(0);

pub type ShaderMap = HashMap<ShaderId, Shader>;
pub type UniformDefs = HashMap<String, UniformDef>;

#[derive(Clone, Debug)]
pub struct Shader {
    pub id: ShaderId,
    pub name: String,
    pub source: String,
    pub uniform_defs: UniformDefs,
    pub bindings: HashMap<String, u32>,
}

/// Opaque handle to a shader. The ID is exposed for debugging purposes.
/// If you set it manually, you're on your own :)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShaderId(pub u64);

impl std::fmt::Display for ShaderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShaderId({})", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderInstance {
    pub id: ShaderId,
    pub uniforms: HashMap<String, Uniform>,
}

#[derive(Clone, Debug)]
pub enum UniformDef {
    F32(Option<f32>),
    Custom { default_data: Option<Vec<u8>>, wgsl_decl: String },
}

impl UniformDef {
    pub fn to_wgsl(&self) -> &'static str {
        match self {
            UniformDef::F32(_) => "f32",
            UniformDef::Custom { .. } => "X",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Uniform {
    F32(OrderedFloat<f32>),
    Custom(Vec<u8>),
}

static CURRENT_SHADER: Lazy<AtomicRefCell<Option<ShaderInstance>>> =
    Lazy::new(|| AtomicRefCell::new(None));

pub fn set_shader(shader_id: ShaderId) {
    *CURRENT_SHADER.borrow_mut() =
        Some(ShaderInstance { id: shader_id, uniforms: Default::default() });
}

pub fn set_default_shader() {
    *CURRENT_SHADER.borrow_mut() = None;
}

pub fn get_current_shader() -> Option<ShaderInstance> {
    CURRENT_SHADER.borrow().clone()
}

/// Generates a new ShaderId. This is intended for internal use only.
pub fn gen_shader_id() -> ShaderId {
    let id = SHADER_IDS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    info!("Generated ShaderId: {}", id);

    ShaderId(id)
}


pub fn set_uniform_f32(name: impl Into<String>, value: f32) {
    set_uniform(name, Uniform::F32(OrderedFloat(value)));
}

pub fn set_uniform(name: impl Into<String>, value: Uniform) {
    if let Some(shader) = &mut *CURRENT_SHADER.borrow_mut() {
        shader.uniforms.insert(name.into(), value);
    }
}

#[derive(Clone, Debug)]
pub enum ShaderError {
    CompileError(String),
}

pub fn create_shader(
    shaders: &mut ShaderMap,
    name: &str,
    source: &str,
    uniform_defs: UniformDefs,
) -> Result<ShaderId, ShaderError> {
    let id = gen_shader_id();

    if shaders.contains_key(&id) {
        return Err(ShaderError::CompileError(format!(
            "Shader with name '{}' already exists",
            name
        )));
    }

    let mut uniforms_src = String::new();

    let bindings = uniform_defs
        .iter()
        .sorted_by_key(|x| x.0)
        .enumerate()
        .map(|(i, (name, _))| (name.clone(), i as u32))
        .collect::<HashMap<String, u32>>();

    for (name, binding) in bindings.iter() {
        let typ = uniform_defs.get(name).unwrap();

        uniforms_src.push_str(&format!(
            "@group(2) @binding({})
            var<uniform> {}: {};",
            binding,
            name,
            typ.to_wgsl()
        ));
    }

    shaders.insert(id, Shader {
        id,
        name: format!("{} Shader", name),
        source: format!("{}\n{}", uniforms_src, source),
        uniform_defs,
        bindings,
    });

    Ok(id)
}

#[derive(Clone, Debug)]
pub enum RenderTargetId {
    Named(String),
    Generated(u64),
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
