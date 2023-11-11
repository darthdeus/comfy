use comfy::*;

simple_game!("Render Traget Example", GameState, setup, update);

pub struct GameState {
    pub my_shader_id: Option<ShaderId>,
    pub intensity: f32,
}

impl GameState {
    pub fn new(_c: &mut EngineState) -> Self {
        Self { my_shader_id: None, intensity: 2.0 }
    }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {}

const SHADER: &str = "

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var final_color: vec4<f32> = tex * in.color;

    final_color.r = final_color.r;
    final_color.g = 0.0;
    final_color.b = 0.0;

    return final_color;
}
";

fn update(state: &mut GameState, c: &mut EngineContext) {
    if state.my_shader_id.is_none() {
        state.my_shader_id = Some(
            create_shader(
                &mut c.renderer.shaders.borrow_mut(),
                "my-shader",
                // &sprite_shader_from_fragment(SHADER),
                SHADER,
                HashMap::new(),
            )
            .unwrap(),
        )
    }

    let shader_id = state.my_shader_id.unwrap();

    // First draw with a default shader.
    draw_comfy(vec2(-2.0, 0.0), WHITE, 0, splat(1.0));

    // When we switch a shader the uniforms will get their default value
    use_shader(shader_id);

    draw_comfy(vec2(0.0, 0.0), WHITE, 0, splat(1.0));

    draw_comfy(vec2(2.0, 0.0), WHITE, 0, splat(1.0));

    // We can also easily switch back to the default sprite shader.
    use_default_shader();
    draw_comfy(vec2(4.0, 0.0), WHITE, 0, splat(1.0));
}
