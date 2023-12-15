use crate::*;

static RENDER_QUEUES: Lazy<AtomicRefCell<RenderQueues>> =
    Lazy::new(|| AtomicRefCell::new(RenderQueues::default()));

static TEXT_QUEUE: Lazy<AtomicRefCell<Vec<DrawText>>> =
    Lazy::new(|| AtomicRefCell::new(Vec::new()));


#[derive(Default)]
struct RenderQueues {
    data: HashMap<i32, RenderQueue>,
}

#[derive(Default)]
pub struct RenderQueue {
    pub meshes: Vec<MeshDraw>,
}

pub fn consume_text_queue() -> Vec<DrawText> {
    let mut queue = TEXT_QUEUE.borrow_mut();
    let mut new_data = Vec::new();
    std::mem::swap(&mut *queue, &mut new_data);
    new_data
}

pub fn consume_render_queues() -> HashMap<i32, RenderQueue> {
    let mut queues = RENDER_QUEUES.borrow_mut();
    let mut new_data = HashMap::new();
    std::mem::swap(&mut new_data, &mut queues.data);
    new_data
}

pub fn queue_mesh_draw(
    mesh: Mesh,
    texture_params: TextureParams,
    shader: Option<ShaderInstance>,
    render_target: Option<RenderTargetId>,
) {
    RENDER_QUEUES
        .borrow_mut()
        .data
        .entry(mesh.z_index)
        .or_default()
        .meshes
        .push(MeshDraw { mesh, texture_params, shader, render_target });
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
