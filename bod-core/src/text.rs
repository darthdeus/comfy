use crate::*;

pub fn load_ttf_font(_path: &str) -> Result<Font> {
    Ok(Font(0))
}

pub fn load_ttf_font_from_bytes(_bytes: &[u8]) -> Result<Font> {
    Ok(Font(0))
}

pub fn measure_text(
    _text: &str,
    _font: Option<Font>,
    _font_size: u16,
    _font_scale: f32,
) -> TextDimensions {
    TextDimensions { width: 2.0, height: 2.0, offset_y: 0.0 }
}

pub struct TextDimensions {
    pub width: f32,
    pub height: f32,
    pub offset_y: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct Font(pub usize);

#[derive(Clone, Debug)]
pub struct TextParams {
    pub font: egui::FontId,
    pub rotation: f32,
    pub color: Color,
}

impl Default for TextParams {
    fn default() -> TextParams {
        TextParams {
            font: egui::FontId::new(20.0, egui::FontFamily::Monospace),
            color: WHITE,
            rotation: 0.0,
        }
    }
}

pub fn draw_text_ex(
    text: &str,
    // TODO: change this to vec2
    position: Position,
    align: TextAlign,
    params: TextParams,
) {
    let _span = span!("draw_text_ex");

    GLOBAL_STATE.borrow_mut().text_queue.push(DrawText {
        text: text.to_string(),
        position,
        color: params.color,
        font: params.font,
        align,
    });
}

pub fn draw_text(text: &str, position: Vec2, color: Color, align: TextAlign) {
    GLOBAL_STATE.borrow_mut().text_queue.push(DrawText {
        text: text.to_string(),
        position: position.as_world(),
        color,
        font: TextParams::default().font,
        align,
    });
}

pub struct DrawText {
    pub text: String,
    pub position: Position,
    pub font: egui::FontId,
    pub color: Color,
    pub align: TextAlign,
}

#[derive(Copy, Clone, Debug)]
pub enum TextAlign {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}
