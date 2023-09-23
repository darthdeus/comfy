use blobs::*;
use comfy::*;

define_versions!();

pub async fn run() {
    let config = GameConfig {
        game_name: "Physics Example",
        version: version_str(),
        ..Default::default()
    };

    let engine = EngineState::new(config);

    let game = Box::new(ComfyGame::new(engine, GameState::new, setup, update));

    let game = Box::leak(game);

    run_comfy_main_async(game, GameContextBuilder).await;
}

fn main() {
    #[cfg(feature = "color-backtrace")]
    color_backtrace::install();

    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(run());
    }
}

struct GameState {
    pub physics: Rc<RefCell<Physics>>,
}

impl GameState {
    pub fn new(_c: &mut EngineContext) -> Self {
        Self { physics: Rc::new(RefCell::new(Physics::new(Vec2::ZERO, false))) }
    }
}

pub struct GameContext<'a> {
    pub physics: &'a Rc<RefCell<Physics>>,
    pub engine: &'a mut EngineState,
}

#[derive(Copy, Clone, Debug)]
struct GameContextBuilder;

impl ContextBuilder<GameState> for GameContextBuilder {
    type Context<'context> = GameContext<'context>;

    fn make_context<'a, 'b: 'a>(
        &self,
        state: &'b mut GameState,
        engine: &'b mut EngineState,
    ) -> Self::Context<'a> {
        GameContext { physics: &mut state.physics, engine }
    }
}


// pub fn game_context_builder<'a, 'b: 'a>(
//     state: &'b mut GameState,
//     c: &'a mut EngineState,
// ) -> GameContext<'a, 'b> {
//     GameContext { physics: &mut state.physics, engine: c }
// }

// pub fn make_context<'a>(
//     state: &mut GameState,
//     c: &mut EngineContext,
// ) -> GameContext<'a> {

fn setup(_state: &mut GameState, _c: &mut EngineContext) {}

fn update(_c: &mut GameContext) {
    draw_circle(Vec2::ZERO, 0.5, RED, 0);
}

// fn foo<'a>(state: &'a mut GameState, c: &'a mut EngineContext<'a>) {
//     let _c = make_game_context(state, c);
// }

// fn main() {}
