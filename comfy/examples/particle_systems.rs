use comfy::*;

simple_game!("Particle Systems Example", setup, update);

fn setup(c: &mut EngineContext) {
    c.load_texture_from_bytes(
        // Every texture gets a string name later used to reference it.
        "comfy",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/comfy.png"
        )),
        wgpu::AddressMode::ClampToEdge,
    );

    // ParticleSystem's are based off the same principle as singular Particle's, except the
    // automatically handle the spawning.

    c.commands().spawn((
        ParticleSystem::with_spawn_rate(500, 0.01, || {
            Particle {
                texture: texture_id("comfy"),
                position: random_circle(5.0),
                size: splat(1.0),
                // Other than the builtin easing curves, any f32 -> f32 curve
                // will work.
                size_curve: quad_in_out,

                direction: vec2(0.0, 1.0),
                velocity: 10.0,
                velocity_end: 5.0,
                ..Default::default()
            }
        })
        .with_size(vec2(3.0, 8.0)),
        Transform::position(vec2(-10.0, 0.0)),
    ));

    c.commands().spawn((
        ParticleSystem::with_spawn_on_death(300, || {
            Particle {
                texture: texture_id("comfy"),
                position: random_circle(5.0),
                size: splat(0.5),
                size_curve: expo_out,

                angular_velocity: random() * 5.0,
                // Both size and color can be faded.
                fade_type: FadeType::Both,
                color_start: RED,
                color_end: RED,
                ..Default::default()
            }
        }),
        Transform::position(vec2(10.0, 0.0)),
    ));

    c.commands().spawn((
        ParticleSystem::with_spawn_on_death(300, || {
            Particle {
                texture: texture_id("comfy"),
                position: random_circle(5.0),

                direction: random_dir().normalize(),

                velocity: 3.0,
                velocity_end: 10.0,
                lifetime_max: 10.0,
                size: splat(0.5),

                // Both size and color can be faded.
                fade_type: FadeType::None,

                color_start: GREEN,
                color_end: LIME,

                trail: TrailRef::Local(Trail::new(
                    0.1,
                    1.0,
                    5,
                    GREEN,
                    BLACK,
                    50,
                    0.5,
                    5.0,
                    None,
                    None,
                    BlendMode::Additive,
                )),

                update: Some(|p| {
                    // Calculate distance from origin.
                    let current_distance = p.position.length();

                    const DESIRED_RADIUS: f32 = 5.0;

                    // Calculate the difference from the desired radius.
                    let difference = DESIRED_RADIUS - current_distance;

                    // Calculate a direction towards or away from the origin.
                    let orbit_pull = if difference > 0.0 {
                        p.position.normalize_or_zero() // Towards origin.
                    } else {
                        -p.position.normalize_or_zero() // Away from origin.
                    };

                    // Tangent along the orbit.
                    let side_pull = p.position.perp().normalize_or_right();

                    let abs_diff = difference.abs();

                    // Rescale the radius from 0..radius to 0..1 for interpolation
                    let abs_diff_scaled =
                        abs_diff.clamp_scale(0.0..DESIRED_RADIUS, 0.0..1.0);

                    let t = if abs_diff < 1.0 {
                        1.0 - abs_diff_scaled
                    } else {
                        abs_diff_scaled
                    };

                    // Combine the two directions.
                    p.direction =
                        orbit_pull.lerp(side_pull, t).normalize_or_right();
                }),

                ..Default::default()
            }
        }),
        Transform::position(Vec2::ZERO),
    ));
}

fn update(_c: &mut EngineContext) {}
