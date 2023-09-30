use crate::*;

pub struct AssetLoader {
    #[cfg(not(target_arch = "wasm32"))]
    pub thread_pool: rayon::ThreadPool,
    pub current_queue: Arc<Mutex<Option<TextureLoadQueue>>>,

    pub texture_load_queue: Vec<(String, String)>,
    pub sound_load_queue: Vec<(String, String)>,
}

impl AssetLoader {
    pub fn new() -> Self {
        let current_queue = Arc::new(Mutex::new(None::<TextureLoadQueue>));

        Self {
            current_queue,
            #[cfg(not(target_arch = "wasm32"))]
            thread_pool: rayon::ThreadPoolBuilder::new().build().unwrap(),

            texture_load_queue: Vec::new(),
            sound_load_queue: Vec::new(),
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
}

pub struct LoadSoundRequest {
    pub path: String,
    pub handle: Sound,
    pub bytes: Vec<u8>,
}

pub struct LoadRequest {
    pub path: String,
    pub handle: TextureHandle,
    pub bytes: Vec<u8>,
}

pub struct LoadedImage {
    pub path: String,
    pub handle: TextureHandle,
    pub image: image::DynamicImage,
}
