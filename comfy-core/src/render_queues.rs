use crate::*;

static SHADER_UNIFORM_TABLE: Lazy<AtomicRefCell<ShaderUniformTable>> =
    Lazy::new(|| AtomicRefCell::new(ShaderUniformTable::default()));

#[derive(Default)]
pub struct ShaderUniformTable {
    next_id: u32,
    table: Vec<ShaderInstance>,
}

pub fn get_shader_instance(
    id: ShaderInstanceId,
) -> AtomicRef<'static, ShaderInstance> {
    AtomicRef::map(SHADER_UNIFORM_TABLE.borrow(), |x| &x.table[id.0 as usize])
}

/// Represents a set of shader uniform parameters.
///
/// u32 ID is exposed for debugging purposes only, do not modify by hand.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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


#[derive(Default)]
struct RenderQueues {
    data: HashMap<MeshGroupKey, RenderQueue>,
}

#[derive(Default)]
pub struct RenderQueue {
    pub meshes: Vec<MeshDraw>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MeshGroupKey {
    pub z_index: i32,
    pub blend_mode: BlendMode,
    pub shader: Option<ShaderInstanceId>,
    pub render_target: Option<RenderTargetId>,
}

pub fn consume_render_queues() -> HashMap<MeshGroupKey, RenderQueue> {
    let mut queues = RENDER_QUEUES.borrow_mut();
    let mut new_data = HashMap::new();
    std::mem::swap(&mut new_data, &mut queues.data);
    new_data
}

pub fn queue_mesh_draw(mesh: Mesh, blend_mode: BlendMode) {
    // let shader = get_current_shader();
    let render_target = get_current_render_target();

    let shader = None;

    RENDER_QUEUES
        .borrow_mut()
        .data
        .entry(MeshGroupKey {
            z_index: mesh.z_index,
            blend_mode,
            shader,
            render_target,
        })
        .or_default()
        .meshes
        .push(MeshDraw { mesh, blend_mode, shader, render_target });
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
