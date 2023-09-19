use comfy::*;

simple_game!("Shapes Example", update);

fn update(_c: &mut EngineContext) {
    clear_background(Color::rgb8(13, 2, 8));

    let z = 0;
    let size = 1.0;
    let thickness = 0.1;

    draw_circle(vec2(0.0, 0.0), size / 2.0, RED, z);
    draw_circle_outline(vec2(0.0, 2.0), size / 2.0, thickness, RED, z);

    draw_rect(vec2(2.0, 0.0), splat(size), GREEN, z);
    draw_rect_outline(vec2(2.0, 2.0), splat(size), thickness, GREEN, z);
}
