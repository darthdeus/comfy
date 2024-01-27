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
simple_game!("Blood Canvas", setup, update);

fn random_blood() -> Color {
    static BLOOD_COLOR: Color = Color { r: 0.454, g: 0.113, b: 0.19, a: 1.0 }; // "#411d31"
    BLOOD_COLOR.darken(random() * 0.2)
}

struct Player;

fn setup(c: &mut EngineContext) {
    // Load the player texture
    c.load_texture_from_bytes(
        "player",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/chicken.png"
        )),
    );

    // Spawn the player entity and make sure z-index is above the grass
    commands().spawn((
        Transform::position(vec2(25.0, 25.0)),
        Player,
        AnimatedSpriteBuilder::new()
            .z_index(10)
            .add_animation("idle", 0.1, true, AnimationSource::Atlas {
                name: "player".into(),
                offset: ivec2(0, 0),
                step: ivec2(16, 0),
                size: isplat(16),
                frames: 1,
            })
            .add_animation("walk", 0.05, true, AnimationSource::Atlas {
                name: "player".into(),
                offset: ivec2(16, 0),
                step: ivec2(16, 0),
                size: isplat(16),
                frames: 6,
            })
            .build(),
    ));
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

    if is_mouse_button_pressed(MouseButton::Right) {
        blood_canvas_blit_at_pro(
            texture_id("error"),
            mouse_world(),
            None,
            RED.alpha(0.7),
            flip_coin(0.5),
            flip_coin(0.5),
        );
    }

    clear_background(TEAL);

    let dt = delta();

    for (_, (_, animated_sprite, transform)) in
        world().query::<(&Player, &mut AnimatedSprite, &mut Transform)>().iter()
    {
        // Handle movement and animation
        let mut moved = false;
        let speed = 3.0;
        let mut move_dir = Vec2::ZERO;

        if is_key_down(KeyCode::W) {
            move_dir.y += 1.0;
            moved = true;
        }
        if is_key_down(KeyCode::S) {
            move_dir.y -= 1.0;
            moved = true;
        }
        if is_key_down(KeyCode::A) {
            move_dir.x -= 1.0;
            moved = true;
        }
        if is_key_down(KeyCode::D) {
            move_dir.x += 1.0;
            moved = true;
        }

        if moved {
            animated_sprite.flip_x = move_dir.x < 0.0;
            transform.position += move_dir.normalize_or_zero() * speed * dt;
            animated_sprite.play("walk");
        } else {
            animated_sprite.play("idle");
        }

        if is_key_pressed(KeyCode::Space) {
            blood_canvas_blit_quad_draw(
                animated_sprite.to_quad_draw(transform),
            );
        }

        main_camera_mut().center = transform.position;
    }
}
