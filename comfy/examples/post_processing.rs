use comfy::*;

example_game!("Post Processing", setup, update);

fn setup(c: &mut EngineContext) {
    let effect = PostProcessingEffect::new(
        "fun-chromatic-aberration",
        &c.graphics_context.device,
        // The first bind group layout has to be passed by hand for now.
        &[&c.graphics_context.texture_bind_group_layout],
        c.surface_config,
        c.render_texture_format,
        include_wgsl_fragment_shader!("fun-chromatic-aberration.wgsl"),
    );

    c.post_processing_effects.borrow_mut().insert(0, effect);
}

fn update(_c: &mut EngineContext) {
    draw_rect(Vec2::ZERO, splat(5.0), WHITE, 0);
}
