use crate::*;

pub fn load_all_pipelines() -> Option<PipelineMap> {
    None
}

pub fn load_shaders() -> ShaderMap {
    let mut shaders = HashMap::new();
    macro_rules! load_shader {
        ($name:literal) => {
            shaders
                .insert($name.into(), reloadable_wgsl_fragment_shader!($name));
        };
    }

    load_shader!("blit");
    load_shader!("bloom-blur");
    load_shader!("bloom-merge");
    load_shader!("bloom-threshold");
    load_shader!("copy");
    load_shader!("chromatic-aberration");
    load_shader!("darken");
    load_shader!("debug");
    load_shader!("film-grain");
    load_shader!("invert");
    load_shader!("palette");
    load_shader!("red");
    load_shader!("sprite");
    load_shader!("screen-shake");
    load_shader!("tonemapping");

    shaders
}
