use comfy::*;

simple_game!("ECS Topdown Game", setup, update);

struct Player;
struct Grass;

fn setup(c: &mut EngineContext) {
    // Load the grass texture
    c.load_texture_from_bytes(
        "grass",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/grass.png"
        )),
    );

    // Load the player texture
    c.load_texture_from_bytes(
        "player",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/chicken.png"
        )),
    );

    for x in 0..50 {
        for y in 0..50 {
            let variant = random_i32(0, 2);
            // Tile the world with random variant of grass sprite
            c.commands().spawn((
                Sprite::new("grass".to_string(), vec2(1.0, 1.0), 0, WHITE)
                    .with_rect(32 * variant, 0, 32, 32),
                Transform::position(vec2(x as f32, y as f32)),
                Grass,
            ));
        }
    }

    // Spawn the player entity and make sure z-index is above the grass
    c.commands().spawn((
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

fn update(c: &mut EngineContext) {
    clear_background(TEAL);

    let dt = c.delta;

    for (_, (_, animated_sprite, transform)) in c
        .world()
        .query::<(&Player, &mut AnimatedSprite, &mut Transform)>()
        .iter()
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

        main_camera_mut().center = transform.position;
    }
}
