use comfy::*;

simple_game!("Particle Systems Example", setup, update);

fn setup(c: &mut EngineContext) {
    c.commands().spawn((
        ParticleSystem::with_spawn_rate(100, 0.1, || {
            Particle {
                position: random_circle(5.0),
                size: splat(1.0),
                ..Default::default()
            }
        }),
        Transform::position(Vec2::ZERO),
    ));
}

fn update(_c: &mut EngineContext) {}
