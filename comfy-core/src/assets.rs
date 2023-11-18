use crate::*;

type BasePathFn = fn(&str) -> String;

pub static ASSETS: Lazy<AtomicRefCell<Assets>> =
    Lazy::new(|| AtomicRefCell::new(Assets::new()));

pub fn texture_id_safe(id: &str) -> Option<TextureHandle> {
    ASSETS.borrow().textures.get(id).copied()
}

pub fn is_texture_loaded(id: &str) -> bool {
    ASSETS.borrow().textures.contains_key(id)
}

pub struct AssetSource {
    pub dir: &'static include_dir::Dir<'static>,
    pub base_path: BasePathFn,
}

impl AssetSource {
    pub fn load_single_item(
        &self,
        relative_path: &str,
    ) -> std::io::Result<Vec<u8>> {
        if cfg!(any(feature = "ci-release", target_arch = "wasm32")) {
            info!("Embedded {}", relative_path);

            // let file = dir.get_file(&path);
            // queue_load_texture_from_bytes(&path, file.contents()).unwrap()
            // let contents = std::fs::read(&relative_path);
            let file = self.dir.get_file(relative_path).unwrap_or_else(|| {
                panic!("Failed to load {}", relative_path);
            });

            Ok(file.contents().to_vec())
        } else {
            let absolute_path = (self.base_path)(relative_path);

            let absolute_path = std::path::Path::new(&absolute_path)
                .canonicalize()
                .unwrap_or_else(|err| {
                    panic!("Failed to load {} ... {:?}", absolute_path, err)
                })
                .to_string_lossy()
                .to_string();

            info!("File {} ... {}", relative_path, absolute_path);

            let contents = std::fs::read(&absolute_path);
            contents.as_ref().unwrap_or_else(|err| {
                panic!("Failed to load {} ... {:?}", absolute_path, err)
            });
            contents
        }
    }
}

pub fn texture_id_unchecked(id: &str) -> TextureHandle {
    TextureHandle::key_unchecked(id)
}

pub fn texture_id(id: &str) -> TextureHandle {
    if id == "1px" {
        texture_id_safe("1px").expect("1px must be loaded")
    } else {
        texture_id_safe(id).unwrap_or_else(|| {
            if id == "error" {
                for key in ASSETS.borrow().textures.keys().sorted() {
                    println!("{key}");
                }

                panic!("Failed to load error texture with ID = '{}'", id)
            }

            texture_id("error")
        })
    }
}

// TODO: rename to something like "unchecked_id"
pub fn texture_path(path: &str) -> TextureHandle {
    TextureHandle::from_path(path)
}

pub fn sound_id(id: &str) -> Sound {
    ASSETS.borrow().sound_ids.get(id).copied().unwrap_or_else(|| {
        if id == "error" {
            panic!("Failed to load error sound {}", id)
        }

        error!("failed to load sound {}", id);
        sound_id("error")
    })
}

pub struct Assets {
    pub asset_loader: AssetLoader,

    pub textures: HashMap<String, TextureHandle>,
    // pub texture_load_bytes_queue: Vec<String>,
    // TODO: private & fix?
    pub texture_image_map:
        Arc<Mutex<HashMap<TextureHandle, image::DynamicImage>>>,

    pub sound_ids: HashMap<String, Sound>,
    pub sounds: Arc<Mutex<HashMap<Sound, StaticSoundData>>>,
    pub sound_handles: HashMap<Sound, StaticSoundHandle>,
    pub fonts: HashMap<FontHandle, fontdue::Font>,

    pub sound_groups: HashMap<String, Vec<Sound>>,
}

// TODO: hash for name and path separately
// TODO: check both for collisions
impl Assets {
    pub fn new() -> Self {
        let image_map = Arc::new(Mutex::new(HashMap::new()));
        let sounds = Arc::new(Mutex::new(HashMap::new()));

        Self {
            asset_loader: AssetLoader::new(sounds.clone()),

            textures: Default::default(),
            texture_image_map: image_map,

            sound_ids: HashMap::default(),
            sounds,
            sound_handles: HashMap::default(),
            sound_groups: HashMap::default(),

            fonts: HashMap::default(),
        }
    }

    pub fn load_font(&mut self, font: fontdue::Font) -> FontHandle {
        let handle = gen_font_handle();
        self.fonts.insert(handle, font);
        handle
    }

    pub fn load_sound_from_bytes(
        &mut self,
        name: &str,
        bytes: &[u8],
        settings: StaticSoundSettings,
    ) {
        let handle = Sound::from_path(name);

        let data = StaticSoundData::from_cursor(
            std::io::Cursor::new(bytes.to_vec()),
            settings,
        )
        .unwrap();

        self.sound_ids.insert(name.to_string(), handle);
        self.sounds.lock().insert(handle, data);
    }

    pub fn process_asset_queues(&mut self) {
        let _span = span!("process_asset_queues");

        self.asset_loader
            .parse_texture_byte_queue(self.texture_image_map.clone());

        self.asset_loader.sound_tick();

        {
            let _span = span!("start_loading_to_memory");

            const TRY_LOCK: bool = true;

            if TRY_LOCK {
                if let Some(guard) = self.texture_image_map.try_lock() {
                    let loaded_textures =
                        guard.keys().cloned().collect::<HashSet<_>>();

                    self.asset_loader.load_textures_to_memory(
                        loaded_textures,
                        &mut self.textures,
                    );
                }
            } else {
                let loaded_textures = self
                    .texture_image_map
                    .lock()
                    .keys()
                    .cloned()
                    .collect::<HashSet<_>>();

                self.asset_loader.load_textures_to_memory(
                    loaded_textures,
                    &mut self.textures,
                );
            }
        }

        self.asset_loader.load_sounds_to_memory(&mut self.sound_ids);
    }

    // #[deprecated]
    // pub fn process_sound_queue(&mut self) {
    // let sound_queue = self
    //     .sound_load_queue
    //     .drain(..)
    //     .filter(|(key, _relative_path)| {
    //         !self.sounds.contains_key(&Sound::from_path(key))
    //     })
    //     .map(|(key, relative_path)| {
    //         let handle = Sound::from_path(&key);
    //
    //         self.sound_ids.insert(key.to_string(), handle);
    //
    //         if cfg!(any(feature = "ci-release", target_arch = "wasm32")) {
    //             info!("Embedded Sound {}", relative_path);
    //
    //             let file = asset_source
    //                 .dir
    //                 .get_file(&relative_path)
    //                 .unwrap_or_else(|| {
    //                     panic!("Failed to load {}", relative_path);
    //                 });
    //
    //             (relative_path, handle, Ok(file.contents().to_vec()))
    //         } else {
    //             info!("File Sound: {}", relative_path);
    //             let absolute_path =
    //                 (asset_source.base_path)(&relative_path);
    //
    //             let absolute_path = std::path::Path::new(&absolute_path)
    //                 .canonicalize()
    //                 .unwrap()
    //                 .to_string_lossy()
    //                 .to_string();
    //
    //             trace!("Loading absolute path {}", absolute_path);
    //
    //             let contents = std::fs::read(absolute_path);
    //
    //             contents.as_ref().unwrap();
    //
    //             (relative_path, handle, contents)
    //         }
    //     })
    //     .filter_map(|(relative_path, handle, data)| {
    //         if let Ok(data) = data {
    //             Some(LoadSoundRequest {
    //                 path: relative_path,
    //                 handle,
    //                 bytes: data,
    //             })
    //         } else {
    //             error!("Error loading {}", relative_path);
    //             None
    //         }
    //     })
    //     .collect_vec();
    // }

    pub fn handle_name(handle: TextureHandle) -> Option<String> {
        ASSETS.borrow().textures.iter().find_map(|(k, v)| {
            if *v == handle {
                Some(k.clone())
            } else {
                None
            }
        })
    }

    pub fn insert_handle(&mut self, name: &str, handle: TextureHandle) {
        self.textures.insert(name.to_string(), handle);
    }

    pub fn get_texture(&self, key: &str) -> TextureHandle {
        match self.textures.get(key) {
            Some(val) => *val,
            None => {
                error!("Missing {}", key);
                error!("");
                error!("Available:");
                for key in self.textures.keys().sorted() {
                    error!("   {}", key);
                }

                panic!("Unable to load texture {}", key);
            }
        }
    }

    pub fn image_size(handle: TextureHandle) -> Option<UVec2> {
        let assets = ASSETS.borrow();
        let image_map = assets.texture_image_map.lock();

        let image = image_map.get(&handle)?;

        Some(uvec2(image.width(), image.height()))
    }

    pub fn load_image_data(
        path: &str,
        texture: TextureHandle,
    ) -> Option<DynamicImage> {
        let assets = ASSETS.borrow();

        let image_map = assets.texture_image_map.lock();

        let Some(image) = image_map.get(&texture) else {
            let mut messages = vec![];

            if get_time() > 5.0 {
                println!("Loaded image map:\n");

                for (id, _) in image_map.iter() {
                    if let Some((path, _)) =
                        ASSETS.borrow().textures.iter().find(|x| x.1 == id)
                    {
                        messages.push(format!("{}: {:?}", path, id));
                    } else {
                        messages.push(format!("no path: {:?}", id));
                    };
                }

                messages.sort();

                for message in messages.into_iter() {
                    println!("{}", message);
                }

                println!("Failed to load image map {}", path);
                println!();
            }

            return None;
        };

        Some(image.clone())
        // let image = texture.get_texture_data();
    }

    pub fn error_loading_image(path: &str) {
        println!("Loaded textures:\n");

        let mut messages = vec![];

        for (path, id) in ASSETS.borrow().textures.iter() {
            messages.push(format!("{}: {:?}", path, id));
        }

        messages.sort();

        for message in messages.into_iter() {
            println!("{}", message);
        }

        println!("Failed to load textures image {}", path);
    }
}

pub fn load_font_from_bytes(font_bytes: &[u8]) -> FontHandle {
    let font =
        fontdue::Font::from_bytes(font_bytes, fontdue::FontSettings::default())
            .unwrap();

    ASSETS.borrow_mut().load_font(font)
}

pub fn load_multiple_sounds(pairs: Vec<(String, String)>) {
    ASSETS.borrow_mut().asset_loader.queue_load_sounds(pairs);
}

pub fn load_multiple_textures(pairs: Vec<(String, String)>) {
    ASSETS.borrow_mut().asset_loader.queue_load_textures(pairs);
}
