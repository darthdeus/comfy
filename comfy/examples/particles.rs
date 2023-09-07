use comfy::*;

example_game!("Particles Example", setup, update);

fn lerped_color(colors: &[Color], t: f32) -> Color {
    let n = colors.len() - 1;
    let tt = t * n as f32;
    let idx = tt as usize;
    let frac = tt.fract();

    let color1 = colors[idx % colors.len()];
    let color2 = colors[(idx + 1) % colors.len()];

    color1.lerp(color2, frac)
}

// fn setup(c: &mut EngineContext) {
//     c.world_mut()
// }

fn update(c: &mut EngineContext) {
    // We only want to spawn a particle once every 100ms.
    // Comfy provides a comfy way of doing ad-hoc timers with `Cooldowns`.
    //
    // A cooldown is identified by a string key and automatically ticked
    // by the engine.
    if c.cooldowns.borrow_mut().can_use("spawn-particle", 0.1) {

        // Particles are automatically simulated once they're spawned.
        spawn_particle(Particle {
            position: random_circle(5.0),
            size: splat(1.0),
            ..Default::default()
        });
    }
}
