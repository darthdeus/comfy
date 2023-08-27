#![allow(clippy::new_without_default)]

use embi::*;

example_game!("Shapes Example", update);

fn update(_c: &mut EngineContext) {
    draw_circle(Vec2::ZERO, 1.0, RED, 0);
}
