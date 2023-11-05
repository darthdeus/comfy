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

pub fn create_reloadable_shader(
    shaders: &mut ShaderMap,
    name: &str,
    reloadable_source: ReloadableShaderSource,
    uniform_defs: UniformDefs,
) -> Result<ShaderId> {
    let id = create_shader(
        shaders,
        name,
        &reloadable_source.static_source,
        uniform_defs,
    )?;

    watch_shader_path(&reloadable_source.path, id)?;

    Ok(id)
}

pub fn watch_shader_path(
    path: &str,
    shader_id: ShaderId,
) -> notify::Result<()> {
    let path = Path::new(path).to_path_buf();

    let mut hot_reload = HOT_RELOAD.lock();
    hot_reload.watch_path(path.as_path())?;
    hot_reload.shader_paths.insert(path, shader_id);

    Ok(())
}

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

                        for path in event
                            .paths
                            .iter()
                            .filter(|x| !x.to_string_lossy().ends_with('~'))
                        {
                            if let Some(shader_id) = self.shader_paths.get(path)
                            {
                                match std::fs::read_to_string(path) {
                                    Ok(source) => {
                                        let fragment_source =
                                            &sprite_shader_from_fragment(
                                                &source,
                                            );

                                        update_shader(
                                            shaders,
                                            *shader_id,
                                            fragment_source,
                                        );
                                    }

                                    Err(error) => {
                                        error!(
                                            "Error loading a shader at {}: \
                                             {:?}",
                                            path.to_string_lossy(),
                                            error
                                        )
                                    }
                                }
                            } else {
                                error!(
                                    "Trying to reload shader at {} but no \
                                     ShaderId defined for that path. This \
                                     likely means a wrong path was passed to \
                                     `create_reloadable_shader`. Existing \
                                     paths: {:?}",
                                    path.to_string_lossy(),
                                    self.shader_paths
                                );
                            }
                        }
                    }
                }

                Err(err) => eprintln!("Error: {:?}", err),
            }
        }

        reload
    }
}
