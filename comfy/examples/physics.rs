use blobs::*;
use comfy::*;

comfy_game!(
    "Physics Example",
    make_context,
    GameContext,
    GameState,
    setup,
    update
);

struct GameContext<'a> {
    pub engine: &'a mut EngineContext<'a>,
    pub physics: &'a Rc<RefCell<Physics>>,
}

struct GameState {
    pub physics: Rc<RefCell<Physics>>,
}

impl GameState {
    pub fn new(_c: &mut EngineContext) -> Self {
        Self { physics: Rc::new(RefCell::new(Physics::new(Vec2::ZERO, false))) }
    }
}

pub fn make_context<'a>(
    state: &'a mut GameState,
    c: &'a mut EngineContext<'a>,
) -> GameContext<'a> {
    GameContext { physics: &mut state.physics, engine: c }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {}

fn update(_c: &mut GameContext) {
    draw_circle(Vec2::ZERO, 0.5, RED, 0);
}

fn foo<'a>(state: &'a mut GameState, c: &'a mut EngineContext<'a>) {
    let _c = make_context(state, c);
}

// fn main() {}
