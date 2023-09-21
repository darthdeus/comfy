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
        Transform::position(vec2(-5.0, 0.0)),
    ));

    c.commands().spawn((
        ParticleSystem::with_spawn_on_death(300, || {
            Particle {
                texture: texture_id("comfy"),
                position: random_circle(5.0),
                size: splat(0.5),
                size_curve: expo_out,
                // Both size and color can be faded.
                fade_type: FadeType::Both,
                color_start: RED,
                color_end: RED,
                ..Default::default()
            }
        }),
        Transform::position(vec2(5.0, 0.0)),
    ));
}

fn update(_c: &mut EngineContext) {}
