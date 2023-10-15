use comfy::*;

simple_game!("Nice red circle", update);

fn update(_c: &mut EngineContext) {
    draw_circle(vec2(0.0, 0.0), 0.5, RED, 0);

    egui::Window::new("Bloom Config")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -100.0))
        .show(egui(), |ui| {
            ui.checkbox(&mut game_config_mut().bloom_enabled, "Bloom Enabled");
        });
}
