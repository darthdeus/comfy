use comfy::*;

simple_game!("Fragment Shader Example", GameState, setup, update);

pub struct GameState {
    pub my_shader_id: Option<ShaderId>,
    pub intensity: f32,
}

impl GameState {
    pub fn new(_c: &mut EngineContext) -> Self {
        Self { my_shader_id: None, intensity: 1.0 }
    }
}

const FRAG_SHADER: &str = r"
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var final_color: vec4<f32> = tex * in.color;

    // ***************************************************************
    // We can use our uniforms here directly by name. Their WGSL
    // declarations are automatically generated, mapped and checked
    // at runtime by Comfy.
    // ***************************************************************
    final_color.r = final_color.r * intensity;
    final_color.g = 0.0;
    final_color.b = abs(cos(time * 5.0)) * 0.3;

    return final_color;
}
";

fn setup(state: &mut GameState, c: &mut EngineContext) {
    game_config_mut().bloom_enabled = true;

    state.my_shader_id = Some(
        // Shader requires a default value for every uniform
        create_shader(
            &mut c.renderer.shaders.borrow_mut(),
            "my-shader",
            &sprite_shader_from_fragment(FRAG_SHADER),
            hashmap! {
                "time".to_string() => UniformDef::F32(Some(0.0)),
                "intensity".to_string() => UniformDef::F32(Some(1.0)),
            },
        )
        .unwrap(),
    )
}

fn update(state: &mut GameState, _c: &mut EngineContext) {
    // First draw with a default shader.
    draw_comfy(vec2(-2.0, 0.0), WHITE, 0, splat(1.0));

    egui::Window::new("Uniforms")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -100.0))
        .show(egui(), |ui| {
            ui.label("HDR intensity");
            ui.add(egui::Slider::new(&mut state.intensity, 1.0..=5.0));
        });

    // When we switch a shader the uniforms will get their default value
    set_shader(state.my_shader_id.unwrap());

    // We can only set one and then draw and the other uniform will be set
    // to the default value we specified when creating the shader.
    set_uniform("time", Uniform::F32(OrderedFloat(get_time() as f32)));
    set_uniform("intensity", Uniform::F32(OrderedFloat(state.intensity)));
    // draw_comfy(vec2(0.0, 0.0), WHITE, 0, splat(1.0));
    draw_sprite_ex(
        texture_id("_builtin-comfy"),
        vec2(0.0, 0.0),
        WHITE,
        0,
        DrawTextureParams { ..Default::default() },
    );

    // This will set "intensity" while retaining "time" from the previous set in this frame, as
    // expected. None of this should be surprising, other than the fact that we can draw in between
    // `set_uniform` calls and things will _just work_.
    //
    // Note that doing things like this will result in the draw calls not being batched together
    // and instead be done in two separate render passes. This is unavoidable and should be
    // expected, but we're mentioning it here just for extra clarity.
    set_uniform("intensity", Uniform::F32(OrderedFloat(state.intensity * 4.0)));

    draw_comfy(vec2(2.0, 0.0), WHITE, 0, splat(1.0));

    set_default_shader();
    draw_comfy(vec2(4.0, 0.0), WHITE, 0, splat(1.0));
}
