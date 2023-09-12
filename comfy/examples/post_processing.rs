use comfy::*;

example_game!("Post Processing", setup, update);

fn setup(c: &mut EngineContext) {
    c.post_processing_effects.borrow_mut().push(PostProcessingEffect {
        name: (),
        enabled: (),
        render_texture: (),
        bind_group: (),
        pipeline: (),
    })
}

fn update(_c: &mut EngineContext) {
    draw_rect(Vec2::ZERO, splat(5.0), WHITE, 0);
}
