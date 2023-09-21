use comfy::*;

simple_game!("Animated Text Example", update);

fn lerped_color(colors: &[Color], t: f32) -> Color {
    let n = colors.len() - 1;
    let tt = t * n as f32;
    let idx = tt as usize;
    let frac = tt.fract();

    let color1 = colors[idx % colors.len()];
    let color2 = colors[(idx + 1) % colors.len()];

    color1.lerp(color2, frac)
}

fn update(_c: &mut EngineContext) {
    let time = get_time() as f32;
    let colors = [RED, GREEN, BLUE, YELLOW, CYAN];
    let text = "comfy has comfy text rendering with egui";

    let s = time.sin();
    let text_t = rescale(s, -1.0..1.0, 0.0..1.0);
    let text_idx = rescale(s, -1.0..1.0, 0.0..(text.len() + 1) as f32) as usize;

    let color_t = (time * 0.1).fract(); // 0.1 controls the speed of color change
    let color = lerped_color(&colors, color_t);

    draw_text(text, Vec2::ZERO, color, TextAlign::Center);

    draw_text_ex(
        &text[..text_idx.clamp(5, text.len())],
        vec2(0.0, -2.0),
        TextAlign::Center,
        TextParams {
            color,
            // Use egui fonts
            font: egui::FontId::new(
                (10.0 + text_t * 30.0) / egui_scale_factor(),
                egui::FontFamily::Proportional,
            ),
            ..Default::default()
        },
    );
}
