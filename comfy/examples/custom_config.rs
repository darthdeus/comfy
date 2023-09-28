use comfy::*;

simple_game!("Portrait mode", GameState, config, setup, update);

fn config(config: GameConfig) -> GameConfig {
    GameConfig {
        resolution: ResolutionConfig::Physical(600, 600 * 16 / 9),
        ..config
    }
}

pub struct GameState {}

impl GameState {
    pub fn new(_c: &EngineContext) -> Self {
        Self {}
    }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {}

fn update(_state: &mut GameState, _c: &mut EngineContext) {
    draw_text("Comfy likes portrait mode", Vec2::ZERO, PINK, TextAlign::Center);
}
