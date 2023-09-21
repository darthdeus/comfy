use comfy::*;

simple_game!("BITMOB", update);

fn update(c: &EngineContext) {
    draw_circle(Vec2::ZERO, 2.0, PINK, 0);
}
