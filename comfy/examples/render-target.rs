use comfy::*;

simple_game!("Render Traget Example", GameState, setup, update);

pub struct GameState {
    pub my_render_target: Option<RenderTargetId>,
    pub my_shader_id: Option<ShaderId>,
    pub intensity: f32,
}

impl GameState {
    pub fn new(_c: &mut EngineState) -> Self {
        Self { my_shader_id: None, my_render_target: None, intensity: 2.0 }
    }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {}

const SHADER: &str = "
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var final_color: vec4<f32> = tex * in.color;

    final_color.r = final_color.r;
    final_color.g = final_color.g * 0.2;
    final_color.b = final_color.b * 0.2;

    return final_color;
}
";

fn update(state: &mut GameState, c: &mut EngineContext) {
    if state.my_shader_id.is_none() {
        state.my_shader_id = Some(
            create_shader(
                &mut c.renderer.shaders.borrow_mut(),
                "my-shader",
                &sprite_shader_from_fragment(SHADER),
                HashMap::new(),
            )
            .unwrap(),
        );

        state.my_render_target =
            Some(create_render_target(c.renderer, &RenderTargetParams {
                label: "my-render-target".to_string(),
                size: uvec2(128, 128),
                filter_mode: wgpu_types::FilterMode::Nearest,
            }));
    }

    let shader_id = state.my_shader_id.unwrap();
    let render_target_id = state.my_render_target.unwrap();

    use_render_target(render_target_id);

    draw_comfy(vec2(-2.0, 0.0), WHITE, 0, splat(4.0));
    use_shader(shader_id);
    draw_comfy(vec2(0.0, 0.0), WHITE, 0, splat(2.0));
    draw_comfy(vec2(2.0, 0.0), WHITE, 0, splat(1.0));
    use_default_shader();
    draw_comfy(vec2(4.0, 0.0), WHITE, 0, splat(1.0));

    use_default_render_target();

    use_shader(shader_id);
    draw_sprite(
        TextureHandle::RenderTarget(render_target_id),
        vec2(0.0, 5.0),
        WHITE,
        0,
        splat(4.0),
    );
    use_default_shader();
    draw_sprite(
        TextureHandle::RenderTarget(render_target_id),
        vec2(2.0, 2.0),
        WHITE,
        0,
        splat(3.0),
    );
}
