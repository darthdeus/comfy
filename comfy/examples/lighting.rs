use comfy::*;

simple_game!("Lighting Example", setup, update);

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

    c.lighting.ambient_light_intensity = 0.1;
}

fn update(_c: &mut EngineContext) {
    draw_rect(Vec2::ZERO, splat(40.0), DARKRED, 0);
    draw_sprite(texture_id("comfy"), Vec2::ZERO, WHITE, 1, splat(5.0));

    let t = get_time() as f32;

    let t1 = t * 2.0;
    let pos = 3.0 * vec2(t1.cos(), t1.sin());

    draw_light(Light::simple(pos, 2.0, 2.0));
    draw_light(Light::simple(vec2(3.0, 0.0), 8.0, 0.5));
}
