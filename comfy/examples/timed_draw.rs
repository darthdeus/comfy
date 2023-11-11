use comfy::*;

simple_game!("Timed Draw Example", update);

fn update(c: &EngineContext) {
    clear_background(BLACK);

    const TIME: f32 = 3.0;

    egui::Window::new("Timed Draw Example")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -100.0))
        .collapsible(false)
        .title_bar(false)
        .resizable(false)
        .show(egui(), |ui| {
            if ui.button("Click me!").clicked() ||
                is_key_pressed(KeyCode::Space)
            {
                c.draw.borrow_mut().timed(TIME, |_c| {
                    draw_text(
                        &format!("I will be visible for {} seconds.", TIME),
                        Vec2::ZERO,
                        WHITE,
                        TextAlign::Center,
                    );

                    draw_sprite(
                        texture_id("comfy"),
                        Vec2::ZERO - vec2(0.0, 4.0),
                        WHITE,
                        0,
                        splat(5.0),
                    );
                });
            }
        });
}
