use crate::*;

use image::RgbaImage;
use std::fmt::Debug;

#[derive(Debug)]
pub struct CanvasBlock {
    pub image: RgbaImage,
    pub handle: TextureHandle,
    pub modified: bool,
}

pub const CANVAS_BLOCK_SIZE: i32 = 1024;
const BLOCK_SIZE: i32 = CANVAS_BLOCK_SIZE;
const PIXELS_PER_WORLD_UNIT: i32 = 16;

pub const fn blood_block_world_size() -> i32 {
    BLOCK_SIZE / PIXELS_PER_WORLD_UNIT
}

#[derive(Debug)]
pub struct BloodCanvas {
    pub creator: Arc<AtomicRefCell<dyn TextureCreator + Send + Sync + 'static>>,
    pub blocks: HashMap<IVec2, CanvasBlock>,
}

impl BloodCanvas {
    pub fn new(
        creator: Arc<AtomicRefCell<dyn TextureCreator + Send + Sync + 'static>>,
    ) -> Self {
        Self { creator, blocks: HashMap::default() }
    }

    pub fn get_pixel(&mut self, position: Vec2) -> Color {
        let position = position * PIXELS_PER_WORLD_UNIT as f32;
        self.get_pixel_internal(position.x as i32, position.y as i32)
    }

    pub fn set_pixel(&mut self, position: Vec2, color: Color) {
        let position = position * PIXELS_PER_WORLD_UNIT as f32;

        self.set_pixel_internal(position.x as i32, position.y as i32, color)
    }

    fn get_block(&mut self, x: i32, y: i32) -> &mut CanvasBlock {
        let key = ivec2(x, y);

        self.blocks.entry(key).or_insert_with(|| {
            let image = DynamicImage::ImageRgba8(RgbaImage::new(
                BLOCK_SIZE as u32,
                BLOCK_SIZE as u32,
            ))
            .to_rgba8();

            let name = format!("blood-canvas-{}-{}", x, y);

            let handle =
                self.creator.borrow_mut().handle_from_image(&name, &image);

            CanvasBlock { handle, image, modified: false }
        })
    }

    pub fn circle_at_internal(
        &mut self,
        position: Vec2,
        radius: i32,
        pixel_prob: f32,
        color: fn() -> Color,
    ) {
        let position = position * PIXELS_PER_WORLD_UNIT as f32;

        let x = position.x as i32;
        let y = position.y as i32;

        for dx in -radius..radius {
            for dy in -radius..radius {
                if dx * dx + dy * dy < radius * radius && flip_coin(pixel_prob)
                {
                    self.set_pixel_internal(x + dx, y + dy, color());
                }
            }
        }
    }

    fn get_pixel_internal(&mut self, x: i32, y: i32) -> Color {
        let bx = (x as f32 / BLOCK_SIZE as f32).floor() as i32;
        let by = (y as f32 / BLOCK_SIZE as f32).floor() as i32;

        let block = self.get_block(bx, by);

        block.modified = true;
        let px = block.image.get_pixel(
            (x - bx * BLOCK_SIZE) as u32,
            (y - by * BLOCK_SIZE) as u32,
        );

        Into::<Color>::into(*px)
    }

    fn set_pixel_internal(&mut self, x: i32, y: i32, color: Color) {
        let bx = (x as f32 / BLOCK_SIZE as f32).floor() as i32;
        let by = (y as f32 / BLOCK_SIZE as f32).floor() as i32;

        let block = self.get_block(bx, by);

        block.modified = true;
        block.image.put_pixel(
            (x - bx * BLOCK_SIZE) as u32,
            (y - by * BLOCK_SIZE) as u32,
            image::Rgba([
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
                (color.a * 255.0) as u8,
            ]),
        );
    }

    pub fn blit_at_sized(
        &mut self,
        texture: TextureHandle,
        position: Vec2,
        source_rect: Option<IRect>,
        tint: Color,
        dest_size: Vec2,
    ) {
        let assets = ASSETS.borrow_mut();
        let image_map = assets.texture_image_map.lock();

        if let Some(image) = image_map.get(&texture).cloned() {
            drop(image_map);
            drop(assets);

            let source = source_rect.unwrap_or(IRect::new(
                ivec2(0, 0),
                ivec2(image.width() as i32, image.height() as i32),
            ));

            // Calculate scaling factors
            let scale_x = dest_size.x / source.size.x as f32;
            let scale_y = dest_size.y / source.size.y as f32;

            for x in 0..dest_size.x as i32 {
                for y in 0..dest_size.y as i32 {
                    // Determine the corresponding pixel in the source image
                    let src_x =
                        ((x as f32 / scale_x) + source.offset.x as f32) as u32;
                    let src_y =
                        ((y as f32 / scale_y) + source.offset.y as f32) as u32;

                    if src_x < image.width() && src_y < image.height() {
                        let px = image.get_pixel(src_x, src_y);

                        if px.0[3] > 0 {
                            self.set_pixel(
                                position + vec2(x as f32, y as f32) / 16.0,
                                Into::<Color>::into(*px) * tint,
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn blit_at(
        &mut self,
        texture: TextureHandle,
        position: Vec2,
        source_rect: Option<IRect>,
        tint: Color,
        flip_x: bool,
        flip_y: bool,
    ) {
        let tint = tint.to_srgb();

        let assets = ASSETS.borrow_mut();
        let image_map = assets.texture_image_map.lock();

        if let Some(image) = image_map.get(&texture).cloned() {
            drop(image_map);
            drop(assets);

            let rect = source_rect.unwrap_or(IRect::new(
                ivec2(0, 0),
                ivec2(image.width() as i32, image.height() as i32),
            ));

            let size_offset = rect.size.as_vec2() / 2.0;

            for x in 0..rect.size.x {
                for y in 0..rect.size.y {
                    let mut read_x = x + rect.offset.x;
                    let mut read_y = y + rect.offset.y;

                    if flip_x {
                        read_x = rect.offset.x + rect.size.x - x - 1;
                    }

                    if !flip_y {
                        read_y = rect.offset.y + rect.size.y - y - 1;
                    }

                    let src_px = image.get_pixel(read_x as u32, read_y as u32);

                    if src_px.0[3] > 0 {
                        let px_pos = position + vec2(x as f32, y as f32) / 16.0 -
                            size_offset / 16.0;

                        if tint.a < 1.0 {
                            let existing = self.get_pixel(px_pos);

                            let tinted = Into::<Color>::into(*src_px)
                                .linear_space_tint(tint.alpha(1.0));

                            self.set_pixel(
                                px_pos,
                                existing.lerp(tinted, tint.a),
                            );
                        } else {
                            self.set_pixel(
                                px_pos,
                                Into::<Color>::into(*src_px)
                                    .linear_space_tint(tint),
                            );
                        }
                    }
                }
            }
        }
    }
}

pub trait TextureCreator: Debug {
    fn handle_from_size(
        &self,
        name: &str,
        size: UVec2,
        fill: Color,
    ) -> TextureHandle;

    fn handle_from_image(&self, name: &str, image: &RgbaImage)
        -> TextureHandle;

    fn update_texture(&self, image: &RgbaImage, texture: TextureHandle);
    fn update_texture_region(
        &self,
        handle: TextureHandle,
        image: &RgbaImage,
        region: IRect,
    );
}
