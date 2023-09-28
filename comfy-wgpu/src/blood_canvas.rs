use crate::*;
pub use comfy_core::*;

pub static BLOOD_CANVAS: OnceCell<AtomicRefCell<BloodCanvas>> = OnceCell::new();

pub fn blood_canvas_update_and_draw(f: fn(IVec2, &CanvasBlock)) {
    let mut canvas = BLOOD_CANVAS.get().unwrap().borrow_mut();
    let canvas = &mut *canvas;

    for (_, block) in canvas.blocks.iter_mut() {
        if block.modified {
            // info!("updating block at {}", key);
            block.modified = false;

            canvas
                .creator
                .borrow_mut()
                .update_texture(&block.image, block.handle);
        }
    }

    for (key, block) in canvas.blocks.iter() {
        f(*key, block);
    }
}

pub fn blood_canvas_reset() {
    BLOOD_CANVAS.get().unwrap().borrow_mut().blocks = HashMap::default();
}

pub fn blood_circle_at(
    position: Vec2,
    radius: i32,
    pixel_prob: f32,
    color: fn() -> Color,
) {
    BLOOD_CANVAS
        .get()
        .unwrap()
        .borrow_mut()
        .circle_at_internal(position, radius, pixel_prob, color);
}

pub fn blood_canvas_blit_at(
    texture: TextureHandle,
    position: Vec2,
    source_rect: Option<IRect>,
    tint: Color,
) {
    BLOOD_CANVAS.get().unwrap().borrow_mut().blit_at(
        texture,
        position,
        source_rect,
        tint,
    );
}

#[derive(Debug)]
pub struct WgpuTextureCreator {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub layout: Arc<wgpu::BindGroupLayout>,
    pub textures:
        Arc<Mutex<HashMap<TextureHandle, (wgpu::BindGroup, Texture)>>>,
}

impl TextureCreator for WgpuTextureCreator {
    fn handle_from_image(
        &self,
        name: &str,
        image: &DynamicImage,
    ) -> TextureHandle {
        let texture =
            Texture::from_image_uninit(&self.device, image, Some(name))
                .unwrap();

        let bind_group =
            self.device.simple_bind_group(name, &texture, &self.layout);

        let handle = texture_path(name);
        self.textures.lock().insert(handle, (bind_group, texture));

        handle
    }

    // TODO: flip order of params for better readability?
    fn update_texture(&self, image: &DynamicImage, handle: TextureHandle) {
        let size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };

        let textures = self.textures.lock();
        let texture = &textures.get(&handle).unwrap().1;

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            // TODO: check which one is correct
            // &image.as_bytes(),
            &image.to_rgba8(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: None,
            },
            size,
        );
    }
}
