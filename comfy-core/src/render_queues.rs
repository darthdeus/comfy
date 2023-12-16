use crate::*;

static SHADER_UNIFORM_TABLE: Lazy<AtomicRefCell<ShaderUniformTable>> =
    Lazy::new(|| AtomicRefCell::new(ShaderUniformTable::default()));

#[derive(Default)]
pub struct ShaderUniformTable {
    instances: Vec<ShaderInstance>,
}

pub fn clear_shader_uniform_table() {
    SHADER_UNIFORM_TABLE.borrow_mut().instances.clear();
}

pub fn get_shader_instance(
    id: ShaderInstanceId,
) -> AtomicRef<'static, ShaderInstance> {
    AtomicRef::map(SHADER_UNIFORM_TABLE.borrow(), |x| {
        &x.instances[id.0 as usize]
    })
}

pub fn set_uniform(name: impl Into<String>, value: Uniform) {
    if let Some(instance_id) = &mut *CURRENT_SHADER.borrow_mut() {
        let mut table = SHADER_UNIFORM_TABLE.borrow_mut();

        if let Some(instance) = table.instances.get(instance_id.0 as usize) {
            let mut new_instance = instance.clone();
            new_instance.uniforms.insert(name.into(), value);

            table.instances.push(new_instance);

            *instance_id = ShaderInstanceId(table.instances.len() as u32 - 1);
        } else {
            panic!(
    "Current shader instance id is invalid.

                This is likely a bug, \
     please report an issue on https://github.com/darthdeus/comfy/issues with \
     some information on what you did."
);
        }
    } else {
        panic!("Trying to set a uniform with no shader active");
    }
}

static CURRENT_SHADER: Lazy<AtomicRefCell<Option<ShaderInstanceId>>> =
    Lazy::new(|| AtomicRefCell::new(None));

/// Switches to the shader with the given ID. The shader must already exist. To revert back to the
/// default shader simply call `use_default_shader()`.
pub fn use_shader(shader_id: ShaderId) {
    let mut table = SHADER_UNIFORM_TABLE.borrow_mut();

    table
        .instances
        .push(ShaderInstance { id: shader_id, uniforms: Default::default() });

    *CURRENT_SHADER.borrow_mut() =
        Some(ShaderInstanceId(table.instances.len() as u32 - 1));
}

/// Switches back to the default shader.
pub fn use_default_shader() {
    *CURRENT_SHADER.borrow_mut() = None;
}

/// Returns the current `ShaderInstance` if any. Currently intended only for internal use.
pub fn get_current_shader() -> Option<ShaderInstanceId> {
    *CURRENT_SHADER.borrow()
}

use std::{sync::atomic::AtomicU64, collections::BTreeMap};

static SHADER_IDS: AtomicU64 = AtomicU64::new(0);

/// Generates a new ShaderId. This is intended for internal use only.
pub fn gen_shader_id() -> ShaderId {
    let id = SHADER_IDS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    info!("Generated ShaderId: {}", id);

    ShaderId(id)
}

/// Represents a set of shader uniform parameters.
///
/// u32 ID is exposed for debugging purposes only, do not modify by hand.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ShaderInstanceId(pub u32);

static TEXT_QUEUE: Lazy<AtomicRefCell<Vec<DrawText>>> =
    Lazy::new(|| AtomicRefCell::new(Vec::new()));

pub fn consume_text_queue() -> Vec<DrawText> {
    let mut queue = TEXT_QUEUE.borrow_mut();
    let mut new_data = Vec::new();
    std::mem::swap(&mut *queue, &mut new_data);
    new_data
}

static RENDER_QUEUES: Lazy<AtomicRefCell<RenderQueues>> =
    Lazy::new(|| AtomicRefCell::new(RenderQueues::default()));


pub type RenderQueue = Vec<Mesh>;

#[derive(Default)]
struct RenderQueues {
    data: BTreeMap<MeshGroupKey, RenderQueue>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MeshGroupKey {
    pub z_index: i32,
    pub blend_mode: BlendMode,
    pub texture_id: TextureHandle,
    pub shader: Option<ShaderInstanceId>,
    pub render_target: Option<RenderTargetId>,
}

pub fn consume_render_queues() -> BTreeMap<MeshGroupKey, RenderQueue> {
    let mut queues = RENDER_QUEUES.borrow_mut();
    let mut new_data = BTreeMap::new();
    std::mem::swap(&mut new_data, &mut queues.data);
    new_data
}

pub fn queue_mesh_draw(mesh: Mesh, blend_mode: BlendMode) {
    let shader = get_current_shader();
    let render_target = get_current_render_target();
    let white_px = TextureHandle::from_path("1px");

    RENDER_QUEUES
        .borrow_mut()
        .data
        .entry(MeshGroupKey {
            z_index: mesh.z_index,
            blend_mode,
            texture_id: mesh.texture.unwrap_or(white_px),
            shader,
            render_target,
        })
        .or_default()
        .push(mesh);
}

pub fn draw_text_internal(
    text: TextData,
    position: Vec2,
    align: TextAlign,
    pro_params: Option<ProTextParams>,
    params: TextParams,
) {
    TEXT_QUEUE.borrow_mut().push(DrawText {
        text,
        position,
        color: params.color,
        font: params.font,
        align,
        pro_params,
        z_index: params.z_index,
    });
}
