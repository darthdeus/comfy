use comfy::*;

simple_game!("Text Example", update);

fn update(_c: &mut EngineContext) {
    let text = "comfy has comfy text rendering with egui";

    draw_text(text, vec2(0.0, 1.0), WHITE, TextAlign::Center);

    draw_text_ex(
        "with configurable fonts",
        vec2(0.0, -1.0),
        TextAlign::Center,
        TextParams {
            color: RED,
            // Use egui fonts
            font: egui::FontId::new(32.0, egui::FontFamily::Proportional),
            ..Default::default()
        },
    );
}
