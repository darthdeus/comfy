use crate::*;

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
    position: Vec2,
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
        position,
        color,
        font: TextParams::default().font,
        align,
    });
}

pub struct DrawText {
    pub text: String,
    pub position: Vec2,
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
