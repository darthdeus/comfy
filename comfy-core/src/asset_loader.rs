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
    pub wgpu_load_queue: Arc<Mutex<Option<TextureLoadQueue>>>,

    pub texture_load_queue: Vec<(String, String)>,
    pub sound_load_queue: Vec<(String, String)>,

    pub sounds: Arc<Mutex<HashMap<Sound, StaticSoundData>>>,
    pub sound_send: Arc<Mutex<Sender<SoundAssetData>>>,
    pub sound_recv: Arc<Mutex<Receiver<SoundAssetData>>>,

    pub texture_data_send: Arc<Mutex<Sender<TextureAssetData>>>,
    pub texture_data_recv: Arc<Mutex<Receiver<TextureAssetData>>>,

    pub asset_source: Option<AssetSource>,

    pub data_load_queue: Vec<AssetData>,

    pub pending_textures: HashSet<String>,
}

impl AssetLoader {
    pub fn new(sounds: Arc<Mutex<HashMap<Sound, StaticSoundData>>>) -> Self {
        let current_queue = Arc::new(Mutex::new(None::<TextureLoadQueue>));

        let (sound_send, sound_recv) =
            std::sync::mpsc::channel::<SoundAssetData>();

        let (texture_data_send, texture_data_recv) =
            std::sync::mpsc::channel::<TextureAssetData>();

        let texture_data_send = Arc::new(Mutex::new(texture_data_send));
        let texture_data_recv = Arc::new(Mutex::new(texture_data_recv));

        Self {
            wgpu_load_queue: current_queue,
            #[cfg(not(target_arch = "wasm32"))]
            thread_pool: rayon::ThreadPoolBuilder::new().build().unwrap(),

            texture_load_queue: Vec::new(),
            sound_load_queue: Vec::new(),

            sounds,
            sound_send: Arc::new(Mutex::new(sound_send)),
            sound_recv: Arc::new(Mutex::new(sound_recv)),

            texture_data_send,
            texture_data_recv,

            asset_source: None,

            data_load_queue: Vec::new(),

            pending_textures: Default::default(),
        }
    }

    pub fn queue_load_sounds(&mut self, sounds: Vec<(String, String)>) {
        inc_assets_queued(sounds.len());
        self.sound_load_queue.extend(sounds);
    }

    pub fn queue_load_textures(&mut self, textures: Vec<(String, String)>) {
        inc_assets_queued(textures.len());

        for (key, _path) in textures.iter() {
            self.pending_textures.insert(key.clone());
        }

        self.texture_load_queue.extend(textures)
    }

    pub fn parse_texture_byte_queue(
        &mut self,
        texture_image_map: Arc<Mutex<HashMap<TextureHandle, DynamicImage>>>,
    ) {
        let _span = span!("parse_texture_byte_queue");

        while let Ok(request) = self.texture_data_recv.lock().try_recv() {
            let image_map = texture_image_map.clone();
            let wgpu_load_queue = self.wgpu_load_queue.clone();

            let process_image = move || {
                let image = image::load_from_memory(&request.bytes);

                let item = match image {
                    Ok(image) => {
                        image_map.lock().insert(request.handle, image.clone());

                        inc_assets_loaded(1);

                        LoadedImage {
                            path: request.path,
                            handle: request.handle,
                            image,
                        }
                    }
                    Err(err) => {
                        error!("Failed to load {} ... {}", request.path, err);
                        return;
                    }
                };

                wgpu_load_queue.lock().get_or_insert_with(Vec::new).push(item);
            };

            #[cfg(target_arch = "wasm32")]
            process_image();

            #[cfg(not(target_arch = "wasm32"))]
            self.thread_pool.spawn(process_image);
        }
    }

    pub fn sound_tick(&mut self) {
        let _span = span!("sound_tick");

        while let Ok(item) = self.sound_recv.lock().try_recv() {
            let sounds = self.sounds.clone();

            let sound_loop = move || {
                match StaticSoundData::from_cursor(
                    std::io::Cursor::new(item.bytes),
                    StaticSoundSettings::default(),
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

    pub fn load_textures_to_memory(
        &mut self,
        loaded_textures: HashSet<TextureHandle>,
        textures: &mut HashMap<String, TextureHandle>,
    ) {
        let _span = span!("load_textures_to_memory");

        if let Some(asset_source) = self.asset_source.as_ref() {
            for (key, relative_path) in self
                .texture_load_queue
                .drain(..)
                .filter(|(key, _relative_path)| {
                    !loaded_textures.contains(&texture_id_unchecked(key))
                })
            {
                let handle = texture_id_unchecked(&key);

                textures.insert(key, handle);

                if let Ok(bytes) = asset_source.load_single_item(&relative_path)
                {
                    let texture_data =
                        TextureAssetData { path: relative_path, handle, bytes };

                    self.texture_data_send.lock().send(texture_data).log_err();
                } else {
                    report_error(
                        format!("{:?}", handle),
                        format!("Error loading {}", relative_path),
                    );
                }
            }
        } else {
            assert!(
                self.texture_load_queue.is_empty(),
                "AssetSource must be initialized before textures are loaded"
            );
        }
    }

    pub fn load_sounds_to_memory(
        &mut self,
        sound_ids: &mut HashMap<String, Sound>,
    ) {
        let _span = span!("load_sounds_to_memory");

        if let Some(asset_source) = self.asset_source.as_ref() {
            for (key, relative_path) in self.sound_load_queue.drain(..) {
                let handle = Sound::from_path(&key);

                if self.sounds.lock().contains_key(&handle) {
                    continue;
                }

                sound_ids.insert(key.to_string(), handle);

                if let Ok(bytes) = asset_source.load_single_item(&relative_path)
                {
                    let item =
                        SoundAssetData { path: relative_path, handle, bytes };

                    self.sound_send.lock().send(item).log_err();
                } else {
                    error!("Error loading {}", relative_path);
                    continue;
                }
            }
        } else {
            assert!(
                self.sound_load_queue.is_empty(),
                "AssetSource must be initialized before sounds are loaded"
            );
        }
    }
}

pub enum AssetData {
    Sound(SoundAssetData),
    Texture(TextureAssetData),
}

pub struct SoundAssetData {
    pub path: String,
    pub handle: Sound,
    pub bytes: Vec<u8>,
}

pub struct TextureAssetData {
    pub path: String,
    pub handle: TextureHandle,
    pub bytes: Vec<u8>,
}

pub struct LoadedImage {
    pub path: String,
    pub handle: TextureHandle,
    pub image: image::DynamicImage,
}
