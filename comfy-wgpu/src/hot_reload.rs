use std::{
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
};

use crate::*;
use notify::{event::AccessKind, Event, EventKind, RecursiveMode, Watcher};

static HOT_RELOAD: Lazy<Mutex<HotReload>> =
    Lazy::new(|| Mutex::new(HotReload::new()));

#[macro_export]
macro_rules! reloadable_shader_source {
    ($path:literal) => {
        ReloadableShaderSource {
            static_source: sprite_shader_from_fragment(include_str!($path)),
            path: $path.to_string(),
        }
    };
}

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

    watch_shader_path(&reloadable_source.path, id)?;

    Ok(id)
}

pub fn watch_shader_path(
    path: &str,
    shader_id: ShaderId,
) -> notify::Result<()> {
    let path = Path::new(path).canonicalize().unwrap().to_path_buf();

    let mut hot_reload = HOT_RELOAD.lock();
    hot_reload.watch_path(path.as_path())?;
    hot_reload.shader_paths.insert(path, shader_id);

    Ok(())
}

/// Internal use only, checks for shader hot reloads and reloads them if needed.
pub fn maybe_reload_shaders(shaders: &mut ShaderMap) {
    HOT_RELOAD.lock().maybe_reload_shaders(shaders);
}

pub struct HotReload {
    rx: Receiver<Result<Event, notify::Error>>,
    watcher: notify::RecommendedWatcher,
    pub shader_paths: HashMap<PathBuf, ShaderId>,
}

impl HotReload {
    pub fn new() -> Self {
        println!("SHADER HOT RELOADING ENABLED!");

        let (tx, rx) = std::sync::mpsc::channel();

        let watcher =
            notify::RecommendedWatcher::new(tx, Default::default()).unwrap();

        Self { rx, watcher, shader_paths: HashMap::new() }
    }

    pub fn watch_path(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)?;

        Ok(())
    }

    pub fn maybe_reload_shaders(&self, shaders: &mut ShaderMap) -> bool {
        let mut reload = false;

        if let Ok(maybe_event) = self.rx.try_recv() {
            match maybe_event {
                Ok(event) => {
                    let is_close_write = matches!(
                        event.kind,
                        EventKind::Access(AccessKind::Close(
                            notify::event::AccessMode::Write
                        ))
                    );

                    let is_temp = event
                        .paths
                        .iter()
                        .all(|p| p.to_string_lossy().ends_with('~'));

                    if is_close_write && !is_temp {
                        reload = true;

                        self.reload_path_bufs(shaders, &event.paths);
                    }
                }

                Err(err) => eprintln!("Error: {:?}", err),
            }
        }

        reload
    }

    fn reload_path_bufs(&self, shaders: &mut ShaderMap, paths: &[PathBuf]) {
        for path in paths.iter().filter(|x| !x.to_string_lossy().ends_with('~'))
        {
            if let Some(shader_id) = self.shader_paths.get(path) {
                match std::fs::read_to_string(path) {
                    Ok(source) => {
                        let fragment_source =
                            &sprite_shader_from_fragment(&source);

                        checked_update_shader(
                            shaders,
                            *shader_id,
                            fragment_source,
                        );
                    }

                    Err(error) => {
                        error!(
                            "Error loading a shader at {}: {:?}",
                            path.to_string_lossy(),
                            error
                        )
                    }
                }
            } else {
                error!(
                    "Trying to reload shader at {} but no ShaderId defined \
                     for that path. This likely means a wrong path was passed \
                     to `create_reloadable_shader`. Existing paths: {:?}",
                    path.to_string_lossy(),
                    self.shader_paths
                );
            }
        }
    }
}

pub fn check_shader_with_naga(source: &str) -> Result<()> {
    let module = naga::front::wgsl::parse_str(source)?;

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    validator.validate(&module)?;

    Ok(())
}

/// Update the shader source for the given shader ID. This can be used by users who with to
/// implement their own shader hot reloading.
pub fn checked_update_shader(
    shaders: &mut ShaderMap,
    id: ShaderId,
    fragment_source: &str,
) {
    let shader_error_id = format!("{}-shader", id);

    if let Some(shader) = shaders.shaders.get_mut(&id) {
        let final_source = build_shader_source(
            fragment_source,
            &shader.bindings,
            &shader.uniform_defs,
        );

        match check_shader_with_naga(&final_source) {
            Ok(()) => {
                clear_error(shader_error_id);
                shader.source = final_source;
            }
            Err(err) => {
                report_error(shader_error_id, format!("SHADER ERROR: {}", err));
                error!("SHADER COMPILE ERROR:\n{:?}", err);
            }
        }
    }
}
