use crate::*;

pub struct CombatText {
    pub position: Vec2,
    pub text: String,
    pub color: Color,
    pub size: f32,
}

impl CombatText {
    pub fn new(position: Vec2, text: String, color: Color, size: f32) -> Self {
        Self { position, text, color, size }
    }
}

pub fn spawn_combat_text(
    commands: &mut CommandBuffer,
    text: String,
    color: Color,
    size: f32,
    position: Vec2,
) {
    commands.spawn((
        CombatText::new(position, text, color, size),
        DespawnAfter(COMBAT_TEXT_LIFETIME),
    ));
}

pub fn combat_text_system(c: &mut EngineContext) {
    for (_, (combat_text, lifetime)) in
        c.world.borrow_mut().query_mut::<(&mut CombatText, &DespawnAfter)>()
    {
        let progress =
            (COMBAT_TEXT_LIFETIME - lifetime.0) / COMBAT_TEXT_LIFETIME;

        // let dims = measure_text(
        //     &combat_text.text,
        //     Some(c.font),
        //     font_size,
        //     font_scale,
        // );

        let off = 0.5;
        // TODO: re-center
        let pos = combat_text.position + vec2(0.0, 1.0) * progress + off;
        // let pos = pos - vec2(dims.width / 2.0, dims.height / 2.0 + off);

        // let screen_pos = world_to_screen(pos) / egui_scale_factor();

        draw_text_ex(
            &combat_text.text,
            pos,
            // screen_pos.x,
            // screen_pos.y,
            // pos.x,
            // pos.y,
            TextAlign::Center,
            TextParams {
                font: egui::FontId::new(16.0, egui::FontFamily::Proportional),
                color: combat_text.color,
                ..Default::default()
            },
        );
    }
}
