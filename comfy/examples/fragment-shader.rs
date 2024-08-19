// use comfy::*;
//
// simple_game!("Fragment Shader Example", GameState, setup, update);
//
// pub struct GameState {
//     pub my_shader_id: Option<ShaderId>,
//     pub intensity: f32,
// }
//
// impl GameState {
//     pub fn new(_c: &mut EngineState) -> Self {
//         Self { my_shader_id: None, intensity: 2.0 }
//     }
// }
//
// fn setup(_state: &mut GameState, _c: &mut EngineContext) {
//     game_config_mut().bloom_enabled = true;
// }
//
// fn update(state: &mut GameState, c: &mut EngineContext) {
//     if state.my_shader_id.is_none() {
//         state.my_shader_id = Some(
//             // Comfy now supports shader hot reloading. We'll create a simple shader and provide
//             // both `static_source` which would be used in release builds allowing the game to be
//             // shipped with shaders embedded in the binary, as well as a path for hot reloading,
//             // which will be watched by Comfy and hot-reloaded on change.
//             //
//             // Note that currently hot reloading an invalid shader will log the error in the
//             // terminal, but will automatically fall back to the previous shader that compiled.
//             create_reloadable_sprite_shader(
//                 &mut c.renderer.shaders.borrow_mut(),
//                 "my-shader",
//                 ReloadableShaderSource {
//                     static_source: include_str!("fragment-shader.wgsl")
//                         .to_string(),
//                     path: "comfy/examples/fragment-shader.wgsl".to_string(),
//                 },
//                 // Uniforms can have default values. When we switch to this shader we'll have to
//                 // set all the uniforms that don't have a default value before drawing anything
//                 // using the shader, otherwise we'll get a crash.
//                 //
//                 // In this case we don't provide a default for "time" as we'll set it every frame,
//                 // but we will provide a value for "intensity" just to showcase how this would
//                 // work.
//                 //
//                 // Experiment with this to learn what happens in different scenarios!
//                 //
//                 // If you change "intensity" default to `None` you'll get a crash saying which
//                 // uniform was missing a value.
//                 hashmap! {
//                     "time".to_string() => UniformDef::F32(None),
//                     "intensity".to_string() => UniformDef::F32(Some(1.0)),
//                 },
//             )
//             .unwrap(),
//         )
//     }
//
//     let shader_id = state.my_shader_id.unwrap();
//
//     // First draw with a default shader.
//     draw_comfy(vec2(-2.0, 0.0), WHITE, 0, splat(1.0));
//
//     egui::Window::new("Uniforms")
//         .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -100.0))
//         .show(egui(), |ui| {
//             ui.label("HDR intensity");
//             ui.add(egui::Slider::new(&mut state.intensity, 1.0..=5.0));
//         });
//
//     // When we switch a shader the uniforms will get their default value
//     use_shader(shader_id);
//
//     let time = get_time() as f32;
//
//     // We can only set one and then draw and the other uniform will be set
//     // to the default value we specified when creating the shader.
//     set_uniform_f32("time", time);
//
//     draw_comfy(vec2(0.0, 0.0), WHITE, 0, splat(1.0));
//
//     // This will set "intensity" while retaining "time" from the previous set in this frame, as
//     // expected. None of this should be surprising, other than the fact that we can draw in between
//     // `set_uniform` calls and things will _just work_.
//     //
//     // Note that doing things like this will result in the draw calls not being batched together
//     // and instead be done in two separate render passes. This is unavoidable and should be
//     // expected, but we're mentioning it here just for extra clarity.
//     set_uniform_f32("intensity", state.intensity);
//
//     draw_comfy(vec2(2.0, 0.0), WHITE, 0, splat(1.0));
//
//     // We can also easily switch back to the default sprite shader.
//     use_default_shader();
//     draw_comfy(vec2(4.0, 0.0), WHITE, 0, splat(1.0));
// }

fn main() {}
