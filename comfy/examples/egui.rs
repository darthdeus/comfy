use comfy::*;

simple_game!("egui example", update);

fn update(_c: &mut EngineContext) {
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
