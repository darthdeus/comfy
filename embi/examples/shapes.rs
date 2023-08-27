#![allow(clippy::new_without_default)]

use embi::*;

define_versions!();

define_main!(run);

pub async fn run() {
    let config = GameConfig {
        game_name: "Shapes Example",
        version: version_str(),

        lighting: GlobalLightingParams {
            ambient_light_intensity: 1.0,

            ..Default::default()
        },

        ..Default::default()
    };


    let game_state = Box::new(EngineState::new(
        config,
        Box::new(move |_c| Arc::new(Mutex::new(Game::new()))),
    ));

    set_main_camera_zoom(30.0);

    run_embi_main_async(game_state).await;
}

pub struct GameState {}

impl GameState {
    pub fn new(_c: &mut EngineContext) -> Self {
        Self {}
    }
}

pub struct Game {
    pub state: Option<GameState>,
}

impl Game {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl GameLoop for Game {
    fn performance_metrics(&self, _world: &mut World, _ui: &mut egui::Ui) {}

    fn early_update(&mut self, _c: &mut EngineContext) {}

    fn update(&mut self, c: &mut EngineContext) {
        if self.state.is_none() {
            self.state = Some(GameState::new(c));
        }

        if let Some(state) = self.state.as_mut() {
            game_update(state, c);
        }
    }

    fn late_update(&mut self, _c: &mut EngineContext) {}
}

fn game_update(_state: &GameState, _c: &mut EngineContext) {
}
