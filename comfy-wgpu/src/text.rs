use crate::*;

use fontdue::{layout::*, *};
use image::{Rgba, RgbaImage};

#[derive(Debug)]
pub struct Glyph {
    pub metrics: fontdue::Metrics,
    pub bitmap: Vec<u8>,
    pub texture: TextureHandle,
}

// TODO: rename :derp:
pub struct TextHandler {
    pub context: GraphicsContext,
    font: Font,
    layout: Layout,

    glyphs: HashMap<char, Glyph>,
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

        let layout = fontdue::layout::Layout::new(
            fontdue::layout::CoordinateSystem::PositiveYUp,
        );

        let glyphs = HashMap::new();

        // for c in " 0123456789\n\t!@#$%^&*(){}[]<>/,.\\';:\"|ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".chars() {
        // }

        Self { context, font, layout, glyphs }
    }

    pub fn get_glyph(&mut self, c: char) -> TextureHandle {
        if !self.glyphs.contains_key(&c) {
            self.prepare_rasterize(c);
        }

        self.glyphs[&c].texture
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

        let texture = if metrics.width == 0 || metrics.height == 0 {
            texture_id("1px")
        } else {
            let mut image =
                RgbaImage::new(metrics.width as u32, metrics.height as u32);

            for x in 0..metrics.width {
                for y in 0..metrics.height {
                    let i = y * metrics.width + x;

                    // if i + 3 >= bitmap.len() {
                    //     error!(":O weird indexing");
                    //     continue;
                    // }

                    let r = bitmap[i];
                    let g = bitmap[i];
                    let b = bitmap[i];
                    let a = bitmap[i];
                    let pixel = Rgba([r, g, b, a]);
                    image.put_pixel(x as u32, y as u32, pixel);
                }
            }

            // let texture = Texture::from_image(
            //     &self.context.device,
            //     &self.context.queue,
            //     &DynamicImage::ImageRgba8(image),
            //     Some("Glyph Image"),
            //     false,
            // )
            // .unwrap();

            let image = DynamicImage::ImageRgba8(image);

            let handle = self
                .context
                .texture_creator
                .borrow_mut()
                .handle_from_image(&format!("glyph-{}", c), &image);

            // self.context
            //     .texture_creator
            //     .borrow_mut()
            //     .update_texture(&image, handle);

            handle
        };


        let glyph = Glyph { metrics, bitmap, texture };

        self.glyphs.insert(c, glyph);
    }

    pub fn layout_text(
        &mut self,
        text: &str,
        size: f32,
        layout_settings: &LayoutSettings,
    ) -> Vec<GlyphPosition> {
        self.layout.reset(layout_settings);

        let fonts = &[self.font.clone()];

        self.layout.append(fonts, &TextStyle::new(text, size, 0));
        self.layout.glyphs().clone()
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

        self.layout.reset(&LayoutSettings { ..LayoutSettings::default() });

        let fonts = &[self.font.clone()];

        self.layout.append(fonts, &TextStyle::new("Hello\n", 16.0, 0));
        self.layout.append(fonts, &TextStyle::new("\tworld!", 16.0, 0));

        self.layout.glyphs().clone()
    }
}
