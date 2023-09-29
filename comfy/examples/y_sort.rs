use comfy::*;

simple_game!("Sprite Example", setup, update);

fn setup(_c: &mut EngineContext) {
    set_y_sort(0, true);
}

fn update(_c: &mut EngineContext) {
    draw_comfy(Vec2::ZERO, WHITE, 0, splat(5.0));
    draw_comfy(vec2(3.0, (4.0 * get_time()).sin() as f32), RED, 0, splat(3.0));
}
