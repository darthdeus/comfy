use comfy::*;

example_game!("Post Processing", setup, update);

fn setup(c: &mut EngineContext) {
    let name = "fun-chromatic-aberration";
    let shader = include_wgsl_fragment_shader!("fun-chromatic-aberration.wgsl");

    c.insert_post_processing_effect(0, name, shader);
}

fn update(_c: &mut EngineContext) {
    draw_rect(Vec2::ZERO, splat(5.0), WHITE, 0);
}
