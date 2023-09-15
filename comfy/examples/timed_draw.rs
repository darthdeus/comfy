use comfy::*;
use comfy_core::egui::Align;

example_game!("Timed Draw Example", update);

fn update(c: &EngineContext) {
    clear_background(BLACK);

    egui::Window::new("Timed Draw Example")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -100.0))
        .collapsible(false)
        .title_bar(false)
        .resizable(false)
        .show(c.egui, |ui| {
            if ui.button("Click me!").clicked() {
                c.draw.borrow_mut().timed(3.0, |c| {
                    draw_text(
                        "I will be visible for 3 seconds.",
                        Vec2::ZERO,
                        WHITE,
                        TextAlign::Center,
                    );

                    draw_texture_z_ex(
                        texture_id("comfy"),
                        Vec2::ZERO - vec2(0.0, 4.0),
                        WHITE,
                        0,
                        DrawTextureParams {
                            dest_size: Some(splat(5.0).as_world_size()),
                            ..Default::default()
                        },
                    );
                });
            }
        });
}
