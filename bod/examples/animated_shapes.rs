#![allow(clippy::new_without_default)]

use bod::*;

example_game!("Shapes Example", update);

fn update(_c: &mut EngineContext) {
    let time = get_time() as f32;

    clear_background(Color::rgb8(13, 2, 8));

    for (i, color) in [RED, GREEN, BLUE, YELLOW, CYAN].into_iter().enumerate() {
        let center = vec2(i as f32 * 2.0, ((i + 1) as f32 * time).sin());
        let radius = (i + 1) as f32 * 0.75;
        let z_index = i as i32;

        draw_circle(center, radius, color, z_index);

        draw_arc(Vec2::ZERO, i.scal)
    }
}
