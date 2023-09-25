use comfy::*;

simple_game!("Post Processing", setup, update);

fn setup(c: &mut EngineContext) {
    let name = "fun-chromatic-aberration";

    let shader = Shader {
        name: name.to_string(),
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/fun-chromatic-aberration.wgsl"
        ))
        .to_string(),
    };

    c.insert_post_processing_effect(0, name, shader);
}

fn update(_c: &mut EngineContext) {
    draw_rect(Vec2::ZERO, splat(5.0), WHITE, 0);
}
