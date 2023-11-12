use comfy::*;

simple_game!("Single Particle Example", update);

fn update(_c: &mut EngineContext) {
    // We only want to spawn a particle once every 100ms.
    // Comfy provides a comfy way of doing ad-hoc timers with `Cooldowns`.
    //
    // A cooldown is identified by a string key and automatically ticked
    // by the engine.
    if cooldowns().can_use("spawn-particle", 0.1) {
        // Particles are automatically simulated once they're spawned.
        spawn_particle(Particle {
            position: random_circle(5.0),
            size: splat(1.0),
            velocity: 0.0,
            velocity_end: 20.0,
            ..Default::default()
        });
    }
}
