use comfy::*;

simple_game!(
    "Framerate & VSync config options",
    GameState,
    config,
    setup,
    update
);

fn config(config: GameConfig) -> GameConfig {
    GameConfig { vsync_enabled: false, target_framerate: 500, ..config }
}

pub struct GameState {}

impl GameState {
    pub fn new(_c: &EngineContext) -> Self {
        Self {}
    }
}


fn setup(_state: &mut GameState, _c: &mut EngineContext) {
    game_config_mut().dev.show_fps = true;
}

fn update(_state: &mut GameState, _c: &mut EngineContext) {
    // Note the color is multiplied by 5.0 to make it brighter
    // and glow with the bloom effect. This is possible because
    // Comfy supports HDR.
    draw_circle(vec2(0.0, 0.0), 0.5, RED, 0);

    let config = game_config();

    draw_text(
        &format!(
            "VSync: {} ... Target FPS: {} ... Real FPS: {}",
            config.vsync_enabled,
            config.target_framerate,
            get_fps()
        ),
        vec2(0.0, -2.0),
        WHITE,
        TextAlign::Center,
    );
}
