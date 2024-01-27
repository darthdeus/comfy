use crate::*;
pub use comfy_core::*;
use image::{ImageBuffer, RgbaImage};

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
        false,
        false,
    );
}

pub fn blood_canvas_blit_at_sized(
    texture: TextureHandle,
    position: Vec2,
    source_rect: Option<IRect>,
    tint: Color,
    size: Vec2,
) {
    BLOOD_CANVAS.get().unwrap().borrow_mut().blit_at_sized(
        texture,
        position,
        source_rect,
        tint,
        size,
    );
}

// TODO: move this out of blood_canvas
#[derive(Debug)]
pub struct WgpuTextureCreator {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub layout: Arc<wgpu::BindGroupLayout>,
    pub textures: Arc<Mutex<TextureMap>>,
}

impl TextureCreator for WgpuTextureCreator {
    fn handle_from_size(
        &self,
        name: &str,
        size: UVec2,
        color: Color,
    ) -> TextureHandle {
        let buffer =
            ImageBuffer::from_pixel(size.x, size.y, color.to_image_rgba());

        self.handle_from_image(name, &buffer)
    }

    fn handle_from_image(
        &self,
        name: &str,
        image: &RgbaImage,
    ) -> TextureHandle {
        let dims = image.dimensions();
        assert!(dims.0 > 0 && dims.1 > 0);

        let dynamic_image = DynamicImage::ImageRgba8(image.clone());

        let texture =
            // Texture::from_image_uninit(&self.device, image, Some(name))
            Texture::from_image(&self.device, &self.queue, &dynamic_image, Some(name), false)
                .unwrap();

        let bind_group =
            self.device.simple_bind_group(Some(name), &texture, &self.layout);

        let handle = texture_path(name);
        self.textures
            .lock()
            .insert(handle, BindableTexture { bind_group, texture });

        let assets = ASSETS.borrow_mut();
        let mut image_map = assets.texture_image_map.lock();
        image_map.insert(handle, Arc::new(image.clone()));

        handle
    }

    fn update_texture_region(
        &self,
        handle: TextureHandle,
        image: &RgbaImage,
        region: IRect,
    ) {
        // assert_eq!(region.size.x, image.width() as i32);
        // assert_eq!(region.size.y, image.height() as i32);

        let size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };

        let textures = self.textures.lock();
        let texture = &textures.get(&handle).unwrap().texture;

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: region.offset.x as u32,
                    y: region.offset.y as u32,
                    z: 0,
                },
            },
            &image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: None,
            },
            size,
        );
    }

    fn update_texture(&self, image: &RgbaImage, handle: TextureHandle) {
        let size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };

        let textures = self.textures.lock();
        let texture = &textures.get(&handle).unwrap().texture;

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: None,
            },
            size,
        );
    }
}
