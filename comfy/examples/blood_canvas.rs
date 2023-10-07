use comfy::*;

// This example might seem quite random, especially given the name.
//
// The blood canvas is actually something that comes from BITGUN,
// our first Rust game, and something we've ported over between many
// engines (Godot, Unity, Bevy, Macroquad, Comfy).
//
// The way it works is that blood and other debris or permanent "decals"
// can be written to a texture on CPU, and then drawn on top of the level
// as a single sprite at Z_BLOOD_CANVAS.
//
// This is of course not incredibly flexible, but we've used it in a few games
// so far and found it very nice.
//
// The actual "canvas" is not just a single texture, but a grid of textures
// that only get allocated on a need-to basis. Basically, once a pixel is written to
// a texture 1024x1024 in that grid area gets allocated (and later reused).
//
// The implementation is relatively simple right now, but it is something we
// want to expand on, as "permanent debris" is something we enjoy in games,
// as it adds more immersion compared to usual "disappearing particles".
//
// While writing to a CPU texture is not the fastest thing in the world,
// _do not be afraid to call these functions_. In both BITGUN and BITGUN Survivors
// we have "meat particles" which are simple objects that write a blood trail as they fly
// on every frame, and it works just fine.
simple_game!("Blood Canvas", update);

fn random_blood() -> Color {
    static BLOOD_COLOR: Color = Color { r: 0.454, g: 0.113, b: 0.19, a: 1.0 }; // "#411d31"
    BLOOD_COLOR.darken(random() * 0.2)
}

fn update(_c: &mut EngineContext) {
    draw_text(
        "Click anywhere for blooooooooood!",
        Vec2::ZERO,
        WHITE,
        TextAlign::Center,
    );

    if is_mouse_button_pressed(MouseButton::Left) {
        blood_circle_at(mouse_world(), 4, 0.6, random_blood);
    }
}
