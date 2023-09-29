use comfy::*;

simple_game!("Sprite Example", setup, update);

fn setup(_c: &mut EngineContext) {
    set_y_sort(0, true);
}

fn update(_c: &mut EngineContext) {
    draw_comfy(Vec2::ZERO, WHITE, 0, splat(5.0));
}
