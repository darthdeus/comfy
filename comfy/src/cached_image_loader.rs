use crate::*;

static CACHED_LOADER: Lazy<AtomicRefCell<CachedImageLoader>> =
    Lazy::new(|| AtomicRefCell::new(CachedImageLoader::new()));

pub fn cached_loader() -> AtomicRef<'static, CachedImageLoader> {
    CACHED_LOADER.borrow()
}

pub fn cached_loader_mut() -> AtomicRefMut<'static, CachedImageLoader> {
    CACHED_LOADER.borrow_mut()
}

#[derive(Default)]
pub struct CachedImageLoader {
    images: HashMap<String, (egui::TextureHandle, UVec2)>,
}

impl CachedImageLoader {
    pub fn new() -> Self {
        Self { images: HashMap::new() }
    }

    pub fn load_or_err(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> (egui::TextureId, UVec2) {
        self.load(ctx, path).unwrap_or_else(|| self.load(ctx, "error").unwrap())
    }

    pub fn image_or_err(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> egui::TextureId {
        self.cached_load(ctx, path)
            .unwrap_or_else(|| self.cached_load(ctx, "error").unwrap())
    }

    pub fn load(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> Option<(egui::TextureId, UVec2)> {
        // TODO: make cached loader return error id instead of failing
        let mut failed = false;

        if !self.images.contains_key(path) {
            info!("Loading uncached egui image {}", path);

            let texture = texture_id_safe(path).or_else(|| {
                Assets::error_loading_image(path);

                failed = true;

                None
            })?;

            let image = Assets::load_image_data(path, texture)?;
            let (width, height) =
                (image.width() as usize, image.height() as usize);

            let rgba = image;
            let image_data = rgba.as_raw();

            let egui_image = egui::ColorImage::from_rgba_unmultiplied(
                [width, height],
                image_data,
            );

            let handle = ctx.load_texture(
                path,
                egui_image,
                egui::TextureOptions::NEAREST,
            );

            // let texture_id = ctx.add_image(handle);
            // let tex = ctx.load_texture(path, image.clone(),
            // egui::TextureFilter::Linear);

            self.images.insert(
                path.to_string(),
                (handle, uvec2(width as u32, height as u32)),
            );
            // println!(
            //     "does it now contain path? {}",
            //     self.images.contains_key(path)
            // );
        }

        let (image, size) = self.images.get(path).unwrap();

        Some((image.id(), *size))
    }

    pub fn cached_load(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> Option<egui::TextureId> {
        self.load(ctx, path).map(|x| x.0)
    }
}
