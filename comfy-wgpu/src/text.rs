use crate::*;

use etagere::AtlasAllocator;
use fontdue::{layout::*, *};
use image::{Rgba, RgbaImage};

#[derive(Debug)]
pub struct Glyph {
    pub metrics: fontdue::Metrics,
    pub bitmap: Vec<u8>,
    // pub texture: TextureHandle,
    // pub allocation: etagere::Allocation,
    pub rect: IRect,
}

fn make_layout() -> fontdue::layout::Layout {
    fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYUp)
}

pub struct TextRasterizer {
    pub context: GraphicsContext,

    glyphs: HashMap<(FontHandle, OrderedFloat<f32>, char), Glyph>,
    atlas: etagere::AtlasAllocator,

    texture: TextureHandle,

    pub atlas_size: u32,
}

impl TextRasterizer {
    pub fn new(context: GraphicsContext) -> Self {
        let glyphs = HashMap::new();

        const TEXT_ATLAS_SIZE: u32 = 4096;
        let size = uvec2(TEXT_ATLAS_SIZE, TEXT_ATLAS_SIZE);

        let texture = context.texture_creator.borrow_mut().handle_from_size(
            "Font Atlas",
            size,
            TRANSPARENT,
        );

        // TODO: prepare ASCII

        // for c in " 0123456789\n\t!@#$%^&*(){}[]<>/,.\\';:\"|ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".chars() {
        // }

        Self {
            context,
            glyphs,
            atlas: AtlasAllocator::new(etagere::size2(
                size.x as i32,
                size.y as i32,
            )),
            texture,
            atlas_size: TEXT_ATLAS_SIZE,
        }
    }

    pub fn calculate_text_layout(
        &mut self,
        assets: &Assets,
        text: TextData,
        pro_params: ProTextParams,
    ) -> (fontdue::layout::Layout, Rect, Option<Vec<StyledGlyph>>) {
        let (clean_text, styled_glyphs) = match text {
            TextData::Raw(raw_text) => (raw_text, None),
            TextData::Rich(rich_text) => {
                (rich_text.clean_text, Some(rich_text.styled_glyphs))
            }
        };

        let font_handle = pro_params.font;
        let font = assets.fonts.get(&font_handle).unwrap();

        let layout = self.layout_text(
            font,
            &clean_text,
            pro_params.font_size,
            &fontdue::layout::LayoutSettings { ..Default::default() },
        );

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for glyph in layout.glyphs() {
            let glyph_min_x = glyph.x;
            let glyph_min_y = glyph.y;
            let glyph_max_x = glyph.x + glyph.width as f32;
            let glyph_max_y = glyph.y + glyph.height as f32;

            min_x = min_x.min(glyph_min_x);
            min_y = min_y.min(glyph_min_y);
            max_x = max_x.max(glyph_max_x);
            max_y = max_y.max(glyph_max_y);
        }

        let layout_rect =
            Rect::from_min_max(vec2(min_x, min_y), vec2(max_x, max_y));

        (layout, layout_rect, styled_glyphs)
    }

    pub fn get_glyph(
        &mut self,
        font_handle: FontHandle,
        font: &Font,
        font_size: f32,
        c: char,
    ) -> (TextureHandle, IRect) {
        let key = (font_handle, OrderedFloat(font_size), c);
        if !self.glyphs.contains_key(&key) {
            self.prepare_rasterize(font_handle, font, font_size, c);
        }

        (self.texture, self.glyphs[&key].rect)
    }

    pub fn prepare_rasterize(
        &mut self,
        font_handle: FontHandle,
        font: &Font,
        font_size: f32,
        c: char,
    ) {
        let (metrics, bitmap) = font.rasterize(c, font_size);

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

        // println!(
        //     "metrics are {} {} for '{}'",
        //     metrics.width, metrics.height, c
        // );

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

            let image = DynamicImage::ImageRgba8(image).flipv().to_rgba8();

            let pad = 2;

            let allocation = self
                .atlas
                .allocate(etagere::size2(
                    metrics.width as i32 + 2 * pad,
                    metrics.height as i32 + 2 * pad,
                ))
                .unwrap_or_else(|| panic!("FAILED TO FIT GLYPH {}", c));

            if self.atlas.free_space() < self.atlas.allocated_space() {
                let used = self.atlas.free_space();
                let total = self.atlas.size().area();
                info!(
                    "font atlas has {}/{} space ({:.2}%) used",
                    used,
                    total,
                    (1.0 - used as f32 / total as f32) * 100.0
                );
            }

            let rect = allocation.rectangle.to_rect();
            let inset_rect = IRect::new(
                ivec2(rect.origin.x + pad, rect.origin.y + pad),
                ivec2(rect.size.width - 2 * pad, rect.size.height - 2 * pad),
            );
            self.context.texture_creator.borrow_mut().update_texture_region(
                self.texture,
                &image,
                inset_rect,
            );

            let glyph = Glyph { metrics, bitmap, rect: inset_rect };

            self.glyphs
                .insert((font_handle, OrderedFloat(font_size), c), glyph);
        };
    }

    pub fn layout_text(
        &mut self,
        font: &Font,
        text: &str,
        size: f32,
        layout_settings: &LayoutSettings,
    ) -> fontdue::layout::Layout {
        let mut layout = make_layout();
        layout.reset(layout_settings);

        layout
            .append(std::slice::from_ref(font), &TextStyle::new(text, size, 0));
        layout
    }

    #[allow(dead_code)]
    pub fn layout_text_demo(&mut self, font: &Font) -> Vec<GlyphPosition> {
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

        let fonts = std::slice::from_ref(font);

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
