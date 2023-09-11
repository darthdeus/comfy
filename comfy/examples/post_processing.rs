use comfy::*;

example_game!("Post Processing", setup, update);

fn setup(c: &mut EngineContext) {

}

fn update(_c: &mut EngineContext) {
    draw_rect(Vec2::ZERO, splat(5.0), WHITE, 0);
}
