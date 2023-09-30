use crate::*;

pub struct AssetLoader {
    #[cfg(not(target_arch = "wasm32"))]
    pub thread_pool: rayon::ThreadPool,
    pub current_queue: Arc<Mutex<Option<TextureLoadQueue>>>,
}

impl AssetLoader {
    pub fn new() -> Self {
        let current_queue = Arc::new(Mutex::new(None::<TextureLoadQueue>));

        Self {
            current_queue,

            #[cfg(not(target_arch = "wasm32"))]
            thread_pool: rayon::ThreadPoolBuilder::new().build().unwrap(),
        }
    }
}
