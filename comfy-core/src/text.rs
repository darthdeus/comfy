use std::sync::atomic::AtomicU64;

use crate::*;

pub struct DrawText {
    pub text: TextData,
    pub position: Vec2,
    pub font: egui::FontId,
    pub color: Color,
    pub align: TextAlign,
    pub z_index: i32,
    // Temporarily to allow both egui and comfy rasterization
    pub pro_params: Option<ProTextParams>,
}

#[derive(Clone, Debug)]
pub struct TextParams {
    pub font: egui::FontId,
    pub rotation: f32,
    pub color: Color,
    pub z_index: i32,
}

impl Default for TextParams {
    fn default() -> TextParams {
        TextParams {
            font: egui::FontId::new(20.0, egui::FontFamily::Monospace),
            color: WHITE,
            rotation: 0.0,
            z_index: 0,
        }
    }
}

#[doc(hidden)]
pub enum TextData {
    Raw(String),
    Rich(RichText),
}

pub fn draw_text_ex(
    text: &str,
    position: Vec2,
    align: TextAlign,
    params: TextParams,
) {
    let _span = span!("draw_text_ex");

    draw_text_internal(
        TextData::Raw(text.to_string()),
        position,
        align,
        None,
        params,
    );
}

pub fn draw_text(text: &str, position: Vec2, color: Color, align: TextAlign) {
    draw_text_internal(
        TextData::Raw(text.to_string()),
        position,
        align,
        None,
        TextParams { color, ..Default::default() },
    )
}

/// This is a first iteration of Comfy's rich text rendering.
///
/// The API works and is fully usable, but it's going to change in backwards
/// incompatible ways in the future, which is the main reason for the `_experimental`
/// suffix.
///
/// This is not intended as a high performance API for rendering webpages
/// or fullscreen books as fast as possible. The goal is maximizing flexibility
/// and ergonomics of highly stylized text for games.
pub fn draw_text_pro_experimental(
    text: RichText,
    position: Vec2,
    color: Color,
    align: TextAlign,
    font_size: f32,
    font: FontHandle,
    z_index: i32,
) {
    draw_text_internal(
        TextData::Rich(text),
        position,
        align,
        Some(ProTextParams { font_size, font }),
        TextParams { color, z_index, ..Default::default() },
    );
}

static FONT_HANDLE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Internal use only font handle generation.
#[doc(hidden)]
pub fn gen_font_handle() -> FontHandle {
    FontHandle(
        FONT_HANDLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
    )
}

/// Opaque handle to a user font.
///
/// The ID is exposed only for debugging purposes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontHandle(pub u64);

#[doc(hidden)]
/// Temporary while the API stabilizes.
pub struct ProTextParams {
    pub font: FontHandle,
    pub font_size: f32,
}

#[derive(Copy, Clone, Debug)]
pub enum TextAlign {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Copy, Clone, Debug)]
pub struct StyledGlyph {
    #[allow(dead_code)]
    /// The related character.
    ///
    /// We don't really need this, but it's easier for debugging the parser.
    pub char: char,
    /// If true the character will wiggle.
    pub wiggle: bool,
    /// Color override of the character
    pub color: Option<Color>,
}

pub struct RichText {
    pub clean_text: String,
    pub styled_glyphs: Vec<StyledGlyph>,
}

/// Parses a simple subset of markdown-like syntax.
///
/// Users should feel encouraged to build their own syntax for rich text
/// based on their needs. This should only serve as a baseline.
pub fn simple_styled_text(text: &str) -> RichText {
    let mut i = 0;

    let mut clean_text = String::new();
    let mut styled_glyphs = vec![];

    let chars = text.chars().collect_vec();

    while i < chars.len() {
        let mut c = chars[i];

        if c == '*' {
            i += 1;
            if i == chars.len() {
                break;
            }

            c = chars[i];
            styled_glyphs.push(StyledGlyph {
                char: c,
                wiggle: true,
                color: Some(PINK.boost(4.0)),
            });
        } else {
            styled_glyphs.push(StyledGlyph {
                char: c,
                wiggle: false,
                color: None,
            });
        }

        clean_text.push(c);

        i += 1;
    }

    RichText { clean_text, styled_glyphs }
}
