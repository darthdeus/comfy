use crate::*;

use std::{fmt::Debug, num::NonZeroU32};

pub static BLOOD_CANVAS: OnceCell<AtomicRefCell<BloodCanvas>> = OnceCell::new();

pub fn blood_canvas_update_and_draw(f: fn(IVec2, &CanvasBlock)) {
    let mut canvas = BLOOD_CANVAS.get().unwrap().borrow_mut();
    let canvas = &mut *canvas;

    for (_, block) in canvas.blocks.iter_mut() {
        if block.modified {
            // info!("updating block at {}", key);
            block.modified = false;

            canvas.creator.update_texture(&block.image, block.handle);
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

#[derive(Debug)]
pub struct GlowTextureCreator {
    pub gl: Arc<glow::Context>,
    pub textures: Arc<AtomicRefCell<HashMap<TextureHandle, Texture>>>,
}

impl TextureCreator for GlowTextureCreator {
    fn handle_from_image(
        &self,
        name: &str,
        image: &DynamicImage,
    ) -> TextureHandle {
        let texture = Texture::from_image(name, self.gl.clone(), &image);
        let handle = TextureHandle::Raw(texture.texture.0.get().into());

        self.textures.borrow_mut().insert(handle, texture);

        handle
    }

    fn update_texture(&self, image: &DynamicImage, handle: TextureHandle) {
        let TextureHandle::Raw(id) = handle else {
            panic!("Expected TextureHandle::Raw, got {:?}", handle);
        };

        unsafe {
            self.gl.bind_texture(
                glow::TEXTURE_2D,
                Some(glow::NativeTexture(NonZeroU32::new(id as u32).unwrap())),
            );

            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as _,
                image.width() as _,
                image.height() as _,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(image.as_bytes()),
            );
        }
    }
}
// pub struct TiledTexture<T> {
//     pub blocks: HashMap<IVec2, T>,
//     builder: fn() -> T,
// }
//
// impl<T> TiledTexture<T> {
//     pub fn get_block(&mut self, x: i32, y: i32) -> &mut T {
//         let key = ivec2(x, y);
//         if !self.blocks.contains_key(&key) {
//             self.blocks.insert(key, (self.builder)());
//         }
//
//
//         self.blocks.get_mut(&key).unwrap()
//         // match self.blocks.entry(ivec2(x, y)) {
//         //     Entry::Occupied(mut entry) => entry.get_mut(),
//         //     Entry::Vacant(entry) => {
//         //         entry.insert(RgbaImage::new(BLOCK_SIZE, BLOCK_SIZE))
//         //     }
//         // }
//     }
// }
