use comfy::*;

simple_game!("Text Example", setup, update);

fn setup(_c: &mut EngineContext) {
    game_config_mut().bloom_enabled = true;
}

fn update(_c: &mut EngineContext) {
    clear_background(DARKBLUE);

    draw_text_pro(
        "comfy has *c*o*m*f*y *t*e*x*t rendering",
        vec2(-5.0, 1.0),
        WHITE,
        TextAlign::Center,
    );

    draw_text_ex(
        "with both builtin TTF rasterizer and with egui",
        vec2(0.0, -1.0),
        TextAlign::Center,
        TextParams {
            color: YELLOW,
            // Use egui fonts
            font: egui::FontId::new(32.0, egui::FontFamily::Proportional),
            ..Default::default()
        },
    );
}
