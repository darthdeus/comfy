use comfy::*;

simple_game!("egui example", GameState, config, setup, update);

pub struct GameState {}

impl GameState {
    pub fn new(_c: &mut EngineState) -> Self {
        Self {}
    }
}

fn config(config: GameConfig) -> GameConfig {
    GameConfig { wasm_append_id: None, ..config }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {}

fn update(_state: &mut GameState, _c: &mut EngineContext) {
    egui::Window::new("Simple egui window")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(egui(), |ui| {
            if ui.button("hello").hovered() {
                ui.colored_label(RED.egui(), "from egui");
            } else {
                ui.label("from egui");
            }
        });
}
