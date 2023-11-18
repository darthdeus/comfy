use crate::*;

pub struct DrawText {
    pub text: String,
    pub position: Vec2,
    pub font: egui::FontId,
    pub color: Color,
    pub align: TextAlign,
    // Temporarily to allow both egui and comfy rasterization
    pub pro: bool,
}

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

    draw_text_internal(text, position, align, false, params);
}

pub fn draw_text(text: &str, position: Vec2, color: Color, align: TextAlign) {
    draw_text_internal(text, position, align, false, TextParams {
        color,
        ..Default::default()
    })
}

pub fn draw_text_pro(
    text: &str,
    position: Vec2,
    color: Color,
    align: TextAlign,
) {
    draw_text_internal(text, position, align, true, TextParams {
        color,
        ..Default::default()
    });
}

fn draw_text_internal(
    text: &str,
    position: Vec2,
    align: TextAlign,
    pro: bool,
    params: TextParams,
) {
    GLOBAL_STATE.borrow_mut().text_queue.push(DrawText {
        text: text.to_string(),
        position,
        color: params.color,
        font: params.font,
        align,
        pro,
    });
}

#[derive(Copy, Clone, Debug)]
pub enum TextAlign {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}
