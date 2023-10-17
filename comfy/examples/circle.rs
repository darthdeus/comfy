use comfy::*;

simple_game!("Nice red circle", setup, update);

fn setup(_c: &mut EngineContext) {
    // Enable bloom
    game_config_mut().bloom_enabled = true;
}

fn update(_c: &mut EngineContext) {
    // Note the color is multiplied by 5.0 to make it brighter
    // and glow with the bloom effect. This is possible because
    // Comfy supports HDR.
    draw_circle(vec2(0.0, 0.0), 0.5, RED * 5.0, 0);

    draw_text(
        "Nice red glowing circle with the help of HDR bloom",
        vec2(0.0, -2.0),
        WHITE,
        TextAlign::Center,
    );
}
