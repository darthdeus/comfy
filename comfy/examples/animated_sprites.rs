use comfy::*;

simple_game!("Sprite Example", setup, update);

fn setup(c: &mut EngineContext) {
    load_texture_from_engine_bytes(
        &c.renderer.context,
        "player",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/player.png"
        )),
        &mut c.renderer.textures.lock(),
        wgpu::AddressMode::ClampToEdge,
    );
}

fn update(_c: &mut EngineContext) {
    draw_sprite_ex(
        texture_id("player"),
        Vec2::ZERO,
        WHITE,
        0,
        DrawTextureParams {
            dest_size: Some(vec2(3.0, 2.5).as_world_size()),
            ..Default::default()
        },
    );

    let t = get_time() as f32;

    let size = 8;
    let animation_time = 0.4;
    let frame_count = 3;
    let frame_time = animation_time / frame_count as f32;
    let frame = (t / frame_time % frame_count as f32) as i32;

    let src_rect = IRect::new(ivec2(size * frame, 0), isplat(size));
    let dest_size =
        splat(rescale(t.sin(), -1.0..1.0, 1.0..2.0)).as_world_size();

    let params = DrawTextureParams {
        dest_size: Some(dest_size),
        source_rect: Some(src_rect),
        ..Default::default()
    };

    draw_sprite_ex(texture_id("player"), vec2(0.0, 3.0), WHITE, 0, params);

    draw_sprite_ex(
        texture_id("player"),
        vec2(-6.0, 3.0),
        RED,
        0,
        DrawTextureParams { rotation: t.sin() * PI, ..params },
    );

    draw_sprite_ex(
        texture_id("player"),
        vec2(6.0, 3.0),
        BLUE.lerp(YELLOW, t.sin().abs()),
        0,
        DrawTextureParams { rotation: t.sin() * PI * 5.0, ..params },
    );

    for i in 0..6 {
        let off = i as f32 * 0.5;
        let position = vec2(6.0, -6.0 + off * 2.0);
        let pivot = position + splat(off / 3.0);

        let z_index = 0;

        draw_circle(position, 0.1, RED, z_index + 1);
        draw_circle(pivot, 0.1, RED, z_index + 1);
        draw_line(position, pivot, 0.05, RED, z_index + 1);
        draw_circle_outline(position, 0.5, 0.05, RED, z_index + 1);

        draw_sprite_ex(
            texture_id("player"),
            position,
            GREEN.lerp(PINK, t.sin().abs()),
            z_index,
            DrawTextureParams {
                rotation: (t.sin() + 1.0) * PI,
                source_rect: Some(IRect::new(ivec2(0, 0), isplat(size))),
                pivot: Some(pivot),
                dest_size: None,
                ..params
            },
        );
    }
}
