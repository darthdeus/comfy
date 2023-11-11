use comfy::*;

simple_game!("Alpha Sprite", setup, update);

fn setup(c: &mut EngineContext) {
    c.load_texture_from_bytes(
        "dot",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/dot.png"
        )),
    );
}

fn update(_c: &mut EngineContext) {
    clear_background(BLACK);

    draw_sprite(texture_id("dot"), vec2(0.0, 0.0), WHITE, 0, splat(5.0));
    draw_text("Gradient Dot", vec2(0.0, 6.0), BLACK, TextAlign::Center);
}
