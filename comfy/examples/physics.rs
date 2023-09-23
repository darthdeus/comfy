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

    let game = ComfyGame::new(engine);

    run_comfy_main_async(game).await;
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

pub struct GameState {
    pub physics: Physics,
}

impl GameState {
    pub fn new(_c: &mut EngineContext) -> Self {
        Self { physics: Physics::new(Vec2::ZERO, false) }
    }
}

pub struct GameContext<'a, 'b: 'a> {
    pub physics: &'a mut Physics,
    pub engine: &'a mut EngineContext<'b>,
}

pub struct ComfyGame {
    pub engine: EngineState,
    pub state: Option<GameState>,
}

impl ComfyGame {
    pub fn new(engine: EngineState) -> Self {
        Self { state: None, engine }
    }
}

impl GameLoop for ComfyGame {
    fn update(&mut self) {
        let mut c = self.engine.make_context();

        if self.state.is_none() {
            let mut state = GameState::new(&mut c);
            setup(&mut state, &mut c);

            self.state = Some(state);
        }

        if let Some(state) = self.state.as_mut() {
            run_early_update_stages(&mut c);
            run_mid_update_stages(&mut c);

            update(&mut GameContext {
                physics: &mut state.physics,
                engine: &mut c,
            });

            run_late_update_stages(&mut c);
        }
    }

    fn engine(&mut self) -> &mut EngineState {
        &mut self.engine
    }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {}

fn update(_c: &mut GameContext) {
    draw_circle(Vec2::ZERO, 0.5, RED, 0);
}
