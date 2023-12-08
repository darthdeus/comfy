use comfy::*;

// This example is almost exactly the same as the ECS Topdown game example, except using a 2.5D
// perspective.
simple_game!("Perspective ECS Topdown Game", setup, update);

struct Player;
struct Grass;
struct Enemy;

// We'll rotate every sprite by the same amount to make them face the camera
// while the ground plane remains rotated up.
pub static ROT3: AtomicCell<f32> = AtomicCell::new(ROT3_AMOUNT);
// Default amount of 3d rotation.
pub const ROT3_AMOUNT: f32 = PI / 3.0;

pub const Z_MOBS: i32 = 5;

pub struct CameraSettings {
    pub use_camera_override: bool,
    pub offset: Vec2,
    pub eye_z: f32,
    pub fov: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            use_camera_override: true,
            offset: vec2(0.0, -10.0),
            eye_z: 10.0,
            fov: 0.8,
        }
    }
}

pub static CAMERA_SETTINGS: Lazy<AtomicRefCell<CameraSettings>> =
    Lazy::new(|| AtomicRefCell::new(CameraSettings::default()));

fn setup(c: &mut EngineContext) {
    main_camera_mut().matrix_fn = Some(Box::new(|_, center: Vec2| {
        let settings = CAMERA_SETTINGS.borrow();

        let eye = vec3(center.x, center.y, settings.eye_z);
        let up = vec3(0.0, 1.0, 0.0);

        let perspective =
            Mat4::perspective_rh(settings.fov, aspect_ratio(), 0.01, 1000.0);


        let view = Mat4::look_at_rh(
            eye + settings.offset.extend(0.0),
            center.extend(0.0),
            up,
        );

        perspective * view
    }));

    set_y_sort(Z_MOBS, true);

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
            commands().spawn((
                Sprite::new("grass".to_string(), vec2(1.0, 1.0), 0, WHITE)
                    .with_rect(32 * variant, 0, 32, 32),
                Transform::position(vec2(x as f32, y as f32)),
                Grass,
            ));
        }
    }

    let start_pos = vec2(25.0, 25.0);

    // Spawn the player entity and make sure z-index is above the grass
    commands().spawn((
        Transform::position(start_pos),
        Player,
        AnimatedSpriteBuilder::new()
            .z_index(Z_MOBS)
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

    let count = 10;

    for x in -count..count {
        for y in -count..count {
            commands().spawn((
                Transform::position(
                    start_pos + vec2(x as f32, y as f32) + splat(0.5),
                ),
                Sprite::new("player", splat(1.0), 5, BLUE)
                    .with_z_index(Z_MOBS)
                    .with_rect(0, 0, 16, 16),
                // We tag these so that we can query them later.
                Enemy,
            ));
        }
    }
}

fn update(c: &mut EngineContext) {
    clear_background(TEAL);

    if is_key_pressed(KeyCode::F2) {
        let mut camera = main_camera_mut();
        camera.use_matrix_fn = !camera.use_matrix_fn;
        ROT3.store(if camera.use_matrix_fn { ROT3_AMOUNT } else { 0.0 });
    }

    let dt = c.delta;

    let mut player_pos = Vec2::ZERO;

    for (_, (_, animated_sprite, transform)) in
        world().query::<(&Player, &mut AnimatedSprite, &mut Transform)>().iter()
    {
        animated_sprite.rotation_x = ROT3.load();
        player_pos = transform.position;

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

    for (_, (sprite, transform, _enemy)) in
        world().query::<(&mut Sprite, &Transform, &Enemy)>().iter()
    {
        sprite.rotation_x = ROT3.load();
        sprite.flip_x = transform.abs_position.x > player_pos.x;
    }
}
