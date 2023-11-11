use crate::*;
use std::sync::atomic::AtomicU64;

static SHADER_IDS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct ShaderMap {
    pub shaders: HashMap<ShaderId, Shader>,
    pub watched_paths: HashMap<String, ShaderId>,
}

impl ShaderMap {
    pub fn new() -> Self {
        Self { shaders: Default::default(), watched_paths: Default::default() }
    }

    pub fn get(&self, id: ShaderId) -> Option<&Shader> {
        self.shaders.get(&id)
    }

    pub fn insert_shader(&mut self, id: ShaderId, shader: Shader) {
        self.shaders.insert(id, shader);
    }

    pub fn exists(&self, id: ShaderId) -> bool {
        self.shaders.contains_key(&id)
    }
}

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

static CURRENT_RENDER_TARGET: Lazy<AtomicRefCell<Option<RenderTargetId>>> =
    Lazy::new(|| AtomicRefCell::new(None));

pub fn use_render_target(id: RenderTargetId) {
    *CURRENT_RENDER_TARGET.borrow_mut() = Some(id);
}

pub fn use_default_render_target() {
    *CURRENT_RENDER_TARGET.borrow_mut() = None;
}

pub fn get_current_render_target() -> Option<RenderTargetId> {
    CURRENT_RENDER_TARGET.borrow().clone()
}

static CURRENT_SHADER: Lazy<AtomicRefCell<Option<ShaderInstance>>> =
    Lazy::new(|| AtomicRefCell::new(None));

/// Switches to the shader with the given ID. The shader must already exist. To revert back to the
/// default shader simply call `use_default_shader()`.
pub fn use_shader(shader_id: ShaderId) {
    *CURRENT_SHADER.borrow_mut() =
        Some(ShaderInstance { id: shader_id, uniforms: Default::default() });
}

/// Switches back to the default shader.
pub fn use_default_shader() {
    *CURRENT_SHADER.borrow_mut() = None;
}

/// Returns the current `ShaderInstance` if any. Currently intended only for internal use.
pub fn get_current_shader() -> Option<ShaderInstance> {
    CURRENT_SHADER.borrow().clone()
}

/// Generates a new ShaderId. This is intended for internal use only.
pub fn gen_shader_id() -> ShaderId {
    let id = SHADER_IDS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    info!("Generated ShaderId: {}", id);

    ShaderId(id)
}

/// Sets a `f32` uniform value by name. The uniform must exist in the shader.
pub fn set_uniform_f32(name: impl Into<String>, value: f32) {
    set_uniform(name, Uniform::F32(OrderedFloat(value)));
}

pub fn set_uniform(name: impl Into<String>, value: Uniform) {
    if let Some(shader) = &mut *CURRENT_SHADER.borrow_mut() {
        shader.uniforms.insert(name.into(), value);
    }
}

/// Creates a new shader and returns its ID. The `source` parameter should only contain the
/// fragment function, as the rest of the shader is automatically generated.
///
/// `uniform_defs` specifies the uniforms that the shader will use. The keys are the uniform names
/// that will be also automatically generated and can be directly used in the shader. Meaning users
/// don't have to care about WGPU bindings/groups.
///
/// For example, if you have a uniform named `time`, you simply use it as `time` in the shader.
///
/// `ShaderMap` can be obtained from `EngineContext` as `c.renderer.shaders.borrow_mut()`
pub fn create_shader(
    shaders: &mut ShaderMap,
    name: &str,
    source: &str,
    uniform_defs: UniformDefs,
) -> Result<ShaderId> {
    let id = gen_shader_id();

    if !source.contains("@vertex") {
        panic!(
            "Missing @vertex function in shader passed to `create_shader`.

             Did you forget to call `sprite_shader_from_fragment`?"
        );
    }

    if shaders.exists(id) {
        bail!("Shader with name '{}' already exists", name);
    }

    let bindings = uniform_defs_to_bindings(&uniform_defs);

    shaders.insert_shader(id, Shader {
        id,
        name: format!("{} Shader", name),
        source: build_shader_source(source, &bindings, &uniform_defs),
        uniform_defs,
        bindings,
    });

    Ok(id)
}

pub fn uniform_defs_to_bindings(
    uniform_defs: &UniformDefs,
) -> HashMap<String, u32> {
    uniform_defs
        .iter()
        .sorted_by_key(|x| x.0)
        .enumerate()
        .map(|(i, (name, _))| (name.clone(), i as u32))
        .collect::<HashMap<String, u32>>()
}

/// Stores both a static source code for a shader as well as path to its file in development. This
/// is used for automatic shader hot reloading.
pub struct ReloadableShaderSource {
    pub static_source: String,
    pub path: String,
}

pub fn build_shader_source(
    fragment_source: &str,
    bindings: &HashMap<String, u32>,
    uniform_defs: &UniformDefs,
) -> String {
    let mut uniforms_src = String::new();

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

    format!("{}\n{}", uniforms_src, fragment_source)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RenderTargetId {
    Named(&'static str),
    Generated(u64),
}
