use comfy::*;

simple_game!("Fragment Shader Example", GameState, setup, update);

pub struct GameState {
    pub my_shader_id: Option<ShaderId>,
    pub intensity: f32,
}

impl GameState {
    pub fn new(_c: &mut EngineContext) -> Self {
        Self { my_shader_id: None, intensity: 2.0 }
    }
}


fn setup(_state: &mut GameState, _c: &mut EngineContext) {
    game_config_mut().bloom_enabled = true;
}

fn update(state: &mut GameState, c: &mut EngineContext) {
    let shader_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/fragment-shader.wgsl");

    let frag_shader = std::fs::read_to_string(shader_path).unwrap();

    if state.my_shader_id.is_none() {
        state.my_shader_id = Some(
            // Shader requires a default value for every uniform
            create_shader(
                &mut c.renderer.shaders.borrow_mut(),
                "my-shader",
                &sprite_shader_from_fragment(&frag_shader),
                hashmap! {
                    "time".to_string() => UniformDef::F32(Some(0.0)),
                    "intensity".to_string() => UniformDef::F32(Some(1.0)),
                },
            )
            .unwrap(),
        )
    }


    let shader_id = state.my_shader_id.unwrap();

    update_shader(
        &mut c.renderer.shaders.borrow_mut(),
        shader_id,
        &sprite_shader_from_fragment(&frag_shader),
    );

    // First draw with a default shader.
    // draw_comfy(vec2(-2.0, 0.0), WHITE, 0, splat(1.0));

    egui::Window::new("Uniforms")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -100.0))
        .show(egui(), |ui| {
            ui.label("HDR intensity");
            ui.add(egui::Slider::new(&mut state.intensity, 1.0..=5.0));
        });

    // When we switch a shader the uniforms will get their default value
    use_shader(shader_id);

    let time = get_time() as f32;

    // We can only set one and then draw and the other uniform will be set
    // to the default value we specified when creating the shader.
    set_uniform_f32("time", time);

    // draw_comfy(vec2(0.0, 0.0), WHITE, 0, splat(1.0));

    // This will set "intensity" while retaining "time" from the previous set in this frame, as
    // expected. None of this should be surprising, other than the fact that we can draw in between
    // `set_uniform` calls and things will _just work_.
    //
    // Note that doing things like this will result in the draw calls not being batched together
    // and instead be done in two separate render passes. This is unavoidable and should be
    // expected, but we're mentioning it here just for extra clarity.
    set_uniform_f32("intensity", state.intensity);

    draw_comfy(vec2(2.0, 0.0), WHITE, 0, splat(1.0));
    //
    // // We can also easily switch back to the default sprite shader.
    // set_default_shader();
    // draw_comfy(vec2(4.0, 0.0), WHITE, 0, splat(1.0));
}
