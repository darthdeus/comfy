use crate::*;

use etagere::AtlasAllocator;
use fontdue::{layout::*, *};
use image::{Rgba, RgbaImage};

#[derive(Debug)]
pub struct Glyph {
    pub metrics: fontdue::Metrics,
    pub bitmap: Vec<u8>,
    // pub texture: TextureHandle,
    pub allocation: etagere::Allocation,
}

fn make_layout() -> fontdue::layout::Layout {
    fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYUp)
}

// TODO: rename :derp:
pub struct TextHandler {
    pub context: GraphicsContext,
    font: Font,

    glyphs: HashMap<char, Glyph>,
    atlas: etagere::AtlasAllocator,

    texture: TextureHandle,

    pub atlas_size: u32,
}

impl TextHandler {
    pub fn new(context: GraphicsContext) -> Self {
        // let font_data =
        //     include_bytes!("../assets/ArianaVioleta.ttf") as &[u8];
        // let font_data =
        //     include_bytes!("../../assets/ThaleahFat_TTF.ttf") as &[u8];
        let font_data =
            include_bytes!("../../assets/fonts/Orbitron-Black.ttf") as &[u8];

        let font = fontdue::Font::from_bytes(
            font_data,
            fontdue::FontSettings::default(),
        )
        .unwrap();

        let glyphs = HashMap::new();

        const TEXT_ATLAS_SIZE: u32 = 4096;
        let size = uvec2(TEXT_ATLAS_SIZE, TEXT_ATLAS_SIZE);

        let texture = context.texture_creator.borrow_mut().handle_from_size(
            "Font Atlas",
            size,
            TRANSPARENT,
        );

        // for c in " 0123456789\n\t!@#$%^&*(){}[]<>/,.\\';:\"|ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".chars() {
        // }

        Self {
            context,
            font,
            glyphs,
            atlas: AtlasAllocator::new(etagere::size2(
                size.x as i32,
                size.y as i32,
            )),
            texture,
            atlas_size: TEXT_ATLAS_SIZE,
        }
    }

    pub fn get_glyph(
        &mut self,
        c: char,
    ) -> (TextureHandle, etagere::Allocation) {
        if !self.glyphs.contains_key(&c) {
            self.prepare_rasterize(c);
        }

        (self.texture, self.glyphs[&c].allocation)
    }

    pub fn prepare_rasterize(&mut self, c: char) {
        // println!("RASTERIZING: {}", c);
        let (metrics, bitmap) = self.font.rasterize(c, 128.0);

        // if metrics.width > 0 {
        //     bitmap.flip_inplace(metrics.width);
        // }

        let mut rgba_bitmap = vec![];

        for x in bitmap.iter() {
            rgba_bitmap.push(*x);
            rgba_bitmap.push(*x);
            rgba_bitmap.push(*x);
            rgba_bitmap.push(*x);
        }

        println!(
            "metrics are {} {} for '{}'",
            metrics.width, metrics.height, c
        );

        if !(metrics.width == 0 || metrics.height == 0) {
            let mut image =
                RgbaImage::new(metrics.width as u32, metrics.height as u32);

            for x in 0..metrics.width {
                for y in 0..metrics.height {
                    let i = y * metrics.width + x;

                    let v = bitmap[i];
                    let pixel = Rgba([v, v, v, v]);
                    image.put_pixel(x as u32, y as u32, pixel);
                }
            }

            let image = DynamicImage::ImageRgba8(image).flipv();

            let allocation = self
                .atlas
                .allocate(etagere::size2(
                    metrics.width as i32,
                    metrics.height as i32,
                ))
                .unwrap_or_else(|| panic!("FAILED TO FIT GLYPH {}", c));

            info!("still have {} free space", self.atlas.free_space());

            self.context.texture_creator.borrow_mut().update_texture_region(
                self.texture,
                &image,
                allocation.to_irect(),
            );

            // handle

            let glyph = Glyph { metrics, bitmap, allocation };

            self.glyphs.insert(c, glyph);
        };
    }

    pub fn layout_text(
        &mut self,
        text: &str,
        size: f32,
        layout_settings: &LayoutSettings,
    ) -> fontdue::layout::Layout {
        let mut layout = make_layout();
        layout.reset(layout_settings);

        let fonts = &[self.font.clone()];

        layout.append(fonts, &TextStyle::new(text, size, 0));
        layout
    }

    #[allow(dead_code)]
    pub fn layout_text_demo(&mut self) -> Vec<GlyphPosition> {
        // let mut layout = fontdue::layout::Layout::new(
        //     fontdue::layout::CoordinateSystem::PositiveYUp,
        // );
        //
        // layout.reset(&LayoutSettings {
        //     ..LayoutSettings::default()
        // });
        //
        // let fonts = &[self.font.clone()];
        //
        // layout.append(fonts, &TextStyle::new("Hello ", 35.0, 0));
        // layout.append(fonts, &TextStyle::new("world!", 40.0, 0));
        //
        // layout.glyphs().clone()

        let mut layout = make_layout();

        layout.reset(&LayoutSettings { ..LayoutSettings::default() });

        let fonts = &[self.font.clone()];

        layout.append(fonts, &TextStyle::new("Hello\n", 16.0, 0));
        layout.append(fonts, &TextStyle::new("\tworld!", 16.0, 0));

        layout.glyphs().clone()
    }
}

pub trait EtagereRectExtensions {
    fn to_irect(&self) -> IRect;
}

impl EtagereRectExtensions for etagere::Allocation {
    fn to_irect(&self) -> IRect {
        let rect = self.rectangle.to_rect();

        let offset = ivec2(rect.origin.x, rect.origin.y);
        let size = ivec2(rect.size.width, rect.size.height);

        IRect { offset, size }
    }
}