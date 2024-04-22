use comfy::*;

simple_game!("Fullscreen", GameState, config, setup, update);

fn config(config: GameConfig) -> GameConfig {
    GameConfig { fullscreen: true, ..config }
}

pub struct GameState {}

impl GameState {
    pub fn new(_c: &EngineState) -> Self {
        Self {}
    }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {}

fn update(_state: &mut GameState, _c: &mut EngineContext) {
    draw_text("Comfy likes fullscreen", Vec2::ZERO, PINK, TextAlign::Center);
}
