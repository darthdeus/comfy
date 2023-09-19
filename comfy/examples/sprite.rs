use comfy::*;

simple_game!("Sprite Example", setup, update);

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
}

fn update(c: &mut EngineContext) {
    draw_texture_z_ex(
        // Drawing sprites/textures requires a TextureHandle which can be calculated from its
        // string name. This incurs a non-measurable overhead in hashing the string, but saves
        // a lot of boilerplate in user code that would have to store asset handles.
        texture_id("comfy"),
        Vec2::ZERO,
        WHITE,
        0,
        DrawTextureParams {
            dest_size: Some(splat(5.0).as_world_size()),
            ..Default::default()
        },
    );
}
