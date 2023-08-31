use bod::*;

example_game!("Sprite Example", setup, update);

fn setup(c: &mut EngineContext) {
    load_texture_from_engine_bytes(
        c.graphics_context,
        "player",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/player.png"
        )),
        &mut c.textures.lock(),
        wgpu::AddressMode::ClampToEdge,
    );
}

fn update(_c: &mut EngineContext) {
    draw_texture_z_ex(
        texture_id("player"),
        Vec2::ZERO,
        WHITE,
        0,
        DrawTextureParams {
            // dest_size: todo!(),
            // source_rect: todo!(),
            // scroll_offset: todo!(),
            // rotation: todo!(),
            // flip_x: todo!(),
            // flip_y: todo!(),
            // pivot: todo!(),
            // shader: todo!(),
            // blend_mode: todo!(),
            ..Default::default()
        },
    );
}
