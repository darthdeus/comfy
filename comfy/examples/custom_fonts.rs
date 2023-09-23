use comfy::*;

simple_game!("Custom Fonts Example", setup, update);

fn setup(c: &mut EngineContext) {
    c.load_fonts_from_bytes(&[(
        "comfy-font",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/fonts/Orbitron-Regular.ttf"
        )),
    )])
}

fn update(_c: &mut EngineContext) {
    let text = "comfy has comfy text rendering with egui";

    draw_text(text, vec2(0.0, 1.0), WHITE, TextAlign::Center);

    draw_text_ex(
        "with configurable fonts",
        vec2(0.0, 0.0),
        TextAlign::Center,
        TextParams {
            color: RED,
            // Use egui fonts
            font: egui::FontId::new(
                32.0,
                egui::FontFamily::Name("comfy-font".into()),
            ),
            ..Default::default()
        },
    );
}
