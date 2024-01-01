use comfy::*;

simple_game!("Sprite Example", setup, update);

fn setup(c: &mut EngineContext) {
    c.load_texture_from_bytes(
        "player",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/player.png"
        )),
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

    // Sprite alignment using draw_sprite_pro
    for i in 0..9 {
        let step = 1.5;

        let x_off = (i % 3) as f32 * step;
        let y_off = (i / 3) as f32 * step;

        let position = vec2(-8.0 + x_off, -3.0 + y_off);
        let sprite_size = vec2(0.8, 0.8);

        let z_index = 0;

        draw_rect_outline(position, sprite_size, 0.1, RED, 0);
        draw_circle(position, 0.1, RED, z_index + 1);

        draw_sprite_pro(
            texture_id("player"),
            position,
            WHITE,
            0,
            DrawTextureProParams {
                source_rect: Some(src_rect),
                align: match i {
                    0 => SpriteAlign::TopLeft,
                    1 => SpriteAlign::TopCenter,
                    2 => SpriteAlign::TopRight,
                    3 => SpriteAlign::CenterLeft,
                    4 => SpriteAlign::Center,
                    5 => SpriteAlign::CenterRight,
                    6 => SpriteAlign::BottomLeft,
                    7 => SpriteAlign::BottomCenter,
                    8 => SpriteAlign::BottomRight,
                    _ => unreachable!(),
                },
                pivot: Some(vec2(0.0, 0.0)),
                // Rotation and size are applied relative to the alignment point
                size: sprite_size + t.sin().abs() * 0.3,
                rotation: t,
                flip_x: false,
                flip_y: false,
                blend_mode: BlendMode::Alpha,
                rotation_x: 0.0,
                y_sort_offset: 0.0,
            },
        )
    }

    // Using the rotation pivot to rotate around a point
    {
        let position = vec2(0.0, -6.0);
        let pivot = vec2(0.25 * t.sin(), 0.25 * t.cos());

        draw_circle_outline(position, 1.0, 0.1, RED, 0);
        draw_circle(position, 0.1, RED, 1);
        draw_circle(position + pivot, 0.1, GREEN, 1);
        draw_sprite_pro(
            texture_id("player"),
            position,
            WHITE,
            0,
            DrawTextureProParams {
                source_rect: Some(src_rect),
                pivot: Some(pivot),
                rotation: t,
                ..Default::default()
            },
        )
    }
}
