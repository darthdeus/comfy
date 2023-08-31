#![allow(clippy::new_without_default)]

use bod::*;

example_game!("Animated Shapes Example", update);

fn update(_c: &mut EngineContext) {
    let time = get_time() as f32;

    clear_background(Color::rgb8(13, 2, 8));

    let colors = [RED, GREEN, BLUE, YELLOW, CYAN];

    for (i, color) in colors.into_iter().enumerate() {
        let s = (i + 1) as f32;
        let t = s * time;

        let z_index = i as i32;

        draw_circle(
            vec2(i as f32 * 2.0 + 2.0, t.sin() - 2.0),
            s * 0.75,
            color,
            z_index,
        );

        let r = rescale(i, 0..colors.len(), 2..5);
        draw_arc(
            vec2(-s - 2.0, t.sin() - 2.0),
            r,
            PI - t.sin(),
            PI - t.cos(),
            color,
            z_index,
        );

        draw_arc_outline(
            vec2(-0.5, s - 0.5),
            r,
            s / 10.0,
            PI / 4.0 - t.cos().powf(2.0),
            t.sin().powf(2.0) + PI * 3.0 / 4.0,
            // t.cos(),
            color,
            z_index,
        );
    }
}
