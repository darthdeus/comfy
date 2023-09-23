use std::{path::Path, sync::mpsc::Receiver};

use crate::*;
use notify::{event::AccessKind, Event, EventKind, RecursiveMode, Watcher};

pub struct HotReload {
    rx: Receiver<Result<Event, notify::Error>>,
    watcher: notify::RecommendedWatcher,
}

impl HotReload {
    pub fn new() -> Self {
        println!("SHADER HOT RELOADING ENABLED!");

        let (tx, rx) = std::sync::mpsc::channel();

        let watcher =
            notify::RecommendedWatcher::new(tx, Default::default()).unwrap();


        let mut x = Self { rx, watcher };

        x.watch_path(Path::new(&concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/shaders"
        )))
        .unwrap();

        x
    }

    pub fn watch_path(&mut self, path: &Path) -> Result<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)?;
        Ok(())
    }

    pub fn maybe_reload_shaders(&self) -> bool {
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
                        // println!("Got watch {:?}", event);
                    }
                }

                Err(err) => eprintln!("Error: {:?}", err),
            }
        }

        reload
    }
}
