use crate::*;
use std::sync::mpsc::{Receiver, Sender};

pub fn init_asset_source(
    dir: &'static include_dir::Dir<'static>,
    base_path: fn(&str) -> String,
) {
    ASSETS.borrow_mut().asset_loader.asset_source =
        Some(AssetSource { dir, base_path });
}

pub struct AssetLoader {
    #[cfg(not(target_arch = "wasm32"))]
    pub thread_pool: rayon::ThreadPool,
    pub current_queue: Arc<Mutex<Option<TextureLoadQueue>>>,

    pub texture_load_queue: Vec<(String, String)>,
    pub sound_load_queue: Vec<(String, String)>,

    pub sounds: Arc<Mutex<HashMap<Sound, StaticSoundData>>>,
    pub sound_recv: Arc<Mutex<Receiver<LoadSoundRequest>>>,

    pub texture_recv: Arc<Mutex<Receiver<Vec<LoadTextureRequest>>>>,

    pub asset_source: Option<AssetSource>,
}

impl AssetLoader {
    pub fn new(
        sounds: Arc<Mutex<HashMap<Sound, StaticSoundData>>>,
        sound_recv: Receiver<LoadSoundRequest>,
        texture_recv: Arc<Mutex<Receiver<Vec<LoadTextureRequest>>>>,
    ) -> Self {
        let sound_recv = Arc::new(Mutex::new(sound_recv));
        let current_queue = Arc::new(Mutex::new(None::<TextureLoadQueue>));

        Self {
            current_queue,
            #[cfg(not(target_arch = "wasm32"))]
            thread_pool: rayon::ThreadPoolBuilder::new().build().unwrap(),

            texture_load_queue: Vec::new(),
            sound_load_queue: Vec::new(),

            sounds,
            sound_recv,

            texture_recv,

            asset_source: None,
        }
    }

    pub fn queue_load_sounds(&mut self, sounds: Vec<(String, String)>) {
        inc_assets_queued(sounds.len());
        self.sound_load_queue.extend(sounds);
    }

    pub fn queue_load_textures(&mut self, textures: Vec<(String, String)>) {
        inc_assets_queued(textures.len());
        self.texture_load_queue.extend(textures)
    }

    pub fn parse_texture_byte_queue(
        &mut self,
        texture_image_map: Arc<Mutex<HashMap<TextureHandle, DynamicImage>>>,
    ) {
        while let Ok(texture_queue) = self.texture_recv.lock().try_recv() {
            #[cfg(target_arch = "wasm32")]
            let iter = texture_queue.into_iter();
            #[cfg(not(target_arch = "wasm32"))]
            let iter = texture_queue.into_par_iter();

            let image_map = texture_image_map.clone();
            let current_queue = self.current_queue.clone();

            let texture_queue: Vec<_> = iter
                .filter_map(|request| {
                    let image = image::load_from_memory(&request.bytes);

                    match image {
                        Ok(image) => {
                            image_map
                                .lock()
                                .insert(request.handle, image.clone());

                            inc_assets_loaded(1);

                            Some(LoadedImage {
                                path: request.path,
                                handle: request.handle,
                                image,
                            })
                        }
                        Err(err) => {
                            error!(
                                "Failed to load {} ... {}",
                                request.path, err
                            );
                            None
                        }
                    }
                })
                .collect();

            let mut queue = current_queue.lock();

            if let Some(queue) = queue.as_mut() {
                queue.extend(texture_queue);
            } else {
                *queue = Some(texture_queue);
            }
        }
    }

    pub fn sound_tick(&mut self) {
        while let Ok(item) = self.sound_recv.lock().try_recv() {
            let sounds = self.sounds.clone();

            let sound_loop = move || {
                // TODO: do this properly
                let settings = if item.path.contains("music") {
                    StaticSoundSettings::new().loop_region(..)
                } else {
                    StaticSoundSettings::default()
                };

                match StaticSoundData::from_cursor(
                    std::io::Cursor::new(item.bytes),
                    settings,
                ) {
                    Ok(sound) => {
                        trace!("Sound {}", item.path);
                        sounds.lock().insert(item.handle, sound);
                        inc_assets_loaded(1);
                    }
                    Err(err) => {
                        error!(
                            "Failed to parse sound at {}: {:?}",
                            item.path, err
                        );
                    }
                }
            };

            #[cfg(target_arch = "wasm32")]
            sound_loop();

            #[cfg(not(target_arch = "wasm32"))]
            self.thread_pool.spawn(sound_loop);
        }
    }

    pub fn texture_tick(
        &mut self,
        loaded_textures: HashSet<TextureHandle>,
        textures: &mut HashMap<String, TextureHandle>,
        texture_send: &Sender<Vec<LoadTextureRequest>>,
    ) {
        if let Some(asset_source) = self.asset_source.as_ref() {
            let load_requests = self
                .texture_load_queue
                .drain(..)
                .filter(|(key, _relative_path)| {
                    !loaded_textures.contains(&texture_id_unchecked(key))
                })
                .map(|(key, relative_path)| {
                    let handle = texture_id_unchecked(&key);

                    textures.insert(key, handle);

                    if cfg!(any(feature = "ci-release", target_arch = "wasm32"))
                    {
                        info!("Embedded texture {}", relative_path);

                        // let file = dir.get_file(&path);
                        // queue_load_texture_from_bytes(&path, file.contents()).unwrap()
                        // let contents = std::fs::read(&relative_path);
                        let file = asset_source
                            .dir
                            .get_file(&relative_path)
                            .unwrap_or_else(|| {
                                panic!("Failed to load {}", relative_path);
                            });

                        (relative_path, handle, Ok(file.contents().to_vec()))
                    } else {
                        let absolute_path =
                            (asset_source.base_path)(&relative_path);

                        info!(
                            "File texture: {} ... {}",
                            relative_path, absolute_path
                        );

                        let absolute_path =
                            std::path::Path::new(&absolute_path)
                                .canonicalize()
                                .unwrap()
                                .to_string_lossy()
                                .to_string();

                        trace!("Loading absolute path {}", absolute_path);

                        let contents = std::fs::read(absolute_path);

                        contents.as_ref().unwrap();

                        (relative_path, handle, contents)
                    }
                })
                .filter_map(|(relative_path, handle, data)| {
                    if let Ok(data) = data {
                        Some(LoadTextureRequest {
                            path: relative_path,
                            handle,
                            bytes: data,
                        })
                    } else {
                        error!("Error loading {}", relative_path);
                        None
                    }
                })
                .collect_vec();

            let texture_queue = load_requests;

            texture_send.send(texture_queue).log_err();
        } else {
            assert!(
                self.texture_load_queue.is_empty(),
                "AssetSource must be initialized before textures are loaded"
            );
        }
    }
}

pub struct LoadSoundRequest {
    pub path: String,
    pub handle: Sound,
    pub bytes: Vec<u8>,
}

pub struct LoadTextureRequest {
    pub path: String,
    pub handle: TextureHandle,
    pub bytes: Vec<u8>,
}

pub struct LoadedImage {
    pub path: String,
    pub handle: TextureHandle,
    pub image: image::DynamicImage,
}
