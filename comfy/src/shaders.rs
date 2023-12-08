// use miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use crate::*;

/// Similar to `create_shader` but automatically hot reloads the shader on change.
/// Note that `create_reloadable_sprite_shader` will automatically call
/// `sprite_shader_from_fragment`, meaning your source should only contain the fragment part.
///
/// The user needs to provide a `ReloadableShaderSource` which contains the static source to be
/// embedded in the binary, as well as the path to the shader file path for hot reloading.
///
/// The [fragment_shader
/// example](https://github.com/darthdeus/comfy/blob/master/comfy/examples/fragment-shader.rs#L24-L57)
/// contains a full working example of how works.
pub fn create_reloadable_sprite_shader(
    shaders: &mut ShaderMap,
    name: &str,
    reloadable_source: ReloadableShaderSource,
    uniform_defs: UniformDefs,
) -> Result<ShaderId> {
    let id = create_shader(
        shaders,
        name,
        &sprite_shader_from_fragment(&reloadable_source.static_source),
        uniform_defs,
    )?;

    #[cfg(not(target_arch = "wasm32"))]
    watch_shader_path(&reloadable_source.path, id)?;

    Ok(id)
}

// use crate::*;
// use notify::{event::AccessKind, Event, EventKind, RecursiveMode, Watcher};
// use std::{path::Path, sync::mpsc::Receiver};
//
// pub struct FileMaterial {
//     pub vert: String,
//     pub frag: String,
//     pub material: Material,
//     pub params: MaterialParams,
// }
//
// impl FileMaterial {
//     pub fn new(
//         vert_path: &str,
//         frag_path: &str,
//         params: MaterialParams,
//     ) -> Result<Self> {
//         let (vert_src, frag_src) = if cfg!(feature = "ci-release") {
//             info!("Loading shaders {} {}", vert_path, frag_path);
//
//             (
//                 std::fs::read_to_string(vert_path)?,
//                 std::fs::read_to_string(frag_path)?,
//                 // SHARED_ASSET_DIR
//                 //     .get_file(vert_path.replace("assets/", ""))
//                 //     .unwrap()
//                 //     .contents_utf8()
//                 //     .unwrap()
//                 //     .to_string(),
//                 // SHARED_ASSET_DIR
//                 //     .get_file(frag_path.replace("assets/", ""))
//                 //     .unwrap()
//                 //     .contents_utf8()
//                 //     .unwrap()
//                 //     .to_string(),
//             )
//         } else {
//             (
//                 std::fs::read_to_string(vert_path)?,
//                 std::fs::read_to_string(frag_path)?,
//             )
//         };
//
//         let cloned_params = MaterialParams {
//             pipeline_params: params.pipeline_params,
//             uniforms: params.uniforms.clone(),
//             textures: params.textures.clone(),
//         };
//
//         Ok(Self {
//             vert: vert_path.to_string(),
//             frag: frag_path.to_string(),
//             material: load_material(&vert_src, &frag_src, params)?,
//             params: cloned_params,
//         })
//     }
//
//     pub fn reload(&mut self) -> Result<()> {
//         let vert_src = std::fs::read_to_string(&self.vert)?;
//         let frag_src = std::fs::read_to_string(&self.frag)?;
//
//         let cloned_params = MaterialParams {
//             pipeline_params: self.params.pipeline_params,
//             uniforms: self.params.uniforms.clone(),
//             textures: self.params.textures.clone(),
//         };
//
//         self.material = load_material(&vert_src, &frag_src, cloned_params)?;
//         warn!("TODO: unload old pipeline");
//
//         Ok(())
//     }
// }
//
// pub struct Materials {
//     pub map: HashMap<String, FileMaterial>,
// }
//
// impl Materials {
//     pub fn new_default() -> Self {
//         let materials = Self { map: Default::default() };
//
//         // materials
//         //     .load("crt", "crt.vert", "crt.frag", MaterialParams::default())
//         //     .unwrap();
//         //
//         // materials
//         //     .load("out", "sprite.vert", "out.frag", MaterialParams::default())
//         //     .unwrap();
//         //
//         // materials
//         //     .load(
//         //         "bloom",
//         //         "sprite.vert",
//         //         "bloom.frag",
//         //         MaterialParams::default(),
//         //     )
//         //     .unwrap();
//         //
//         // // materials
//         // //     .load(
//         // //         "gaussian_blur",
//         // //         "sprite.vert",
//         // //         "gaussian_blur.frag",
//         // //         MaterialParams::default(),
//         // //     )
//         // //     .unwrap();
//         //
//         // materials
//         //     .load("hdr", "sprite.vert", "hdr.frag", MaterialParams::default())
//         //     .unwrap();
//         //
//         // materials
//         //     .load(
//         //         "color-replacement",
//         //         "sprite.vert",
//         //         "color-replacement.frag",
//         //         MaterialParams {
//         //             uniforms: vec![
//         //                 ("Color1".to_string(), UniformType::Float3),
//         //                 ("Color2".to_string(), UniformType::Float3),
//         //                 ("Color3".to_string(), UniformType::Float3),
//         //                 ("Color4".to_string(), UniformType::Float3),
//         //                 ("Color5".to_string(), UniformType::Float3),
//         //                 ("Color6".to_string(), UniformType::Float3),
//         //                 ("Color7".to_string(), UniformType::Float3),
//         //                 ("Color8".to_string(), UniformType::Float3),
//         //             ],
//         //             ..Default::default()
//         //         },
//         //     )
//         //     .unwrap();
//         //
//         // materials
//         //     .load(
//         //         "blend",
//         //         "sprite.vert",
//         //         "blend.frag",
//         //         MaterialParams {
//         //             textures: vec!["Blend".to_string()],
//         //             uniforms: vec![("Ratio".to_string(), UniformType::Float1)],
//         //             ..Default::default()
//         //         },
//         //     )
//         //     .unwrap();
//         //
//         // // materials
//         // //     .load(
//         // //         "sprite",
//         // //         "sprite.vert",
//         // //         "sprite.frag",
//         // //         MaterialParams {
//         // //             uniforms: vec![
//         // //                 ("Lights".to_string(), UniformType::Float2),
//         // //                 ("Light1".to_string(), UniformType::Float3),
//         // //                 ("Light2".to_string(), UniformType::Float3),
//         // //                 ("Light3".to_string(), UniformType::Float3),
//         // //                 ("Light4".to_string(), UniformType::Float3),
//         // //                 ("Light5".to_string(), UniformType::Float3),
//         // //                 ("Light6".to_string(), UniformType::Float3),
//         // //                 ("ModA".to_string(), UniformType::Float1),
//         // //                 ("ModB".to_string(), UniformType::Float1),
//         // //                 ("ModC".to_string(), UniformType::Float1),
//         // //             ],
//         // //             pipeline_params: PipelineParams {
//         // //                 alpha_blend: Some(BlendState::new(
//         // //                     Equation::Add,
//         // //                     BlendFactor::Zero,
//         // //                     BlendFactor::One,
//         // //                 )),
//         // //                 color_blend: Some(BlendState::new(
//         // //                     Equation::Add,
//         // //                     BlendFactor::Value(BlendValue::SourceAlpha),
//         // //                     BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
//         // //                 )),
//         // //                 ..Default::default() // cull_face: todo!(),
//         // //                                      // front_face_order: todo!(),
//         // //                                      // depth_test: todo!(),
//         // //                                      // depth_write: todo!(),
//         // //                                      // depth_write_offset: todo!(),
//         // //                                      // color_blend: todo!(),
//         // //                                      // stencil_test: todo!(),
//         // //                                      // color_write: todo!(),
//         // //                                      // primitive_type: todo!(),
//         // //             },
//         // //             ..Default::default()
//         // //         },
//         // //     )
//         // //     .unwrap();
//
//         materials
//     }
//
//     pub fn get(&self, key: &str) -> Option<&FileMaterial> {
//         self.map.get(key)
//     }
//
//     pub fn load(
//         &mut self,
//         key: &str,
//         vert: &str,
//         frag: &str,
//         params: MaterialParams,
//     ) -> Result<()> {
//         let vert = format!("assets/shaders/{}", vert);
//         let frag = format!("assets/shaders/{}", frag);
//
//         info!("Loading {}", key);
//
//         let mat = FileMaterial::new(&vert, &frag, params)?;
//         self.map.insert(key.to_string(), mat);
//
//         Ok(())
//     }
//
//     pub fn reload_all(&mut self) -> Result<()> {
//         if !cfg!(feature = "ci-release") {
//             for (_, file_mat) in self.map.iter_mut() {
//                 file_mat.reload()?;
//             }
//
//             info!("All materials reloaded!");
//         }
//
//         Ok(())
//     }
// }
