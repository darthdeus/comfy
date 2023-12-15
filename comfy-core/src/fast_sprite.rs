use crate::*;

pub fn draw_sprite_rot(
    texture: TextureHandle,
    position: Vec2,
    tint: Color,
    z_index: i32,
    rotation: f32,
    dest_size: Vec2,
) {
    let _span = span!("draw_sprite_rot");

    let vertices = simple_rotated_rect(
        position.extend(z_index as f32),
        tint,
        dest_size,
        rotation,
    );

    const QUAD_INDICES_U32: &[u32] = &[0, 2, 1, 0, 3, 2];

    let mesh = Mesh {
        origin: position.extend(z_index as f32),
        vertices: vertices.into(),
        indices: QUAD_INDICES_U32.into(),
        z_index,
        texture: Some(texture),
    };

    draw_mesh_ex(mesh, TextureParams { blend_mode: BlendMode::None });
}

pub fn simple_rotated_rect(
    position: Vec3,
    color: Color,
    dest_size: Vec2,
    rotation: f32,
) -> [SpriteVertex; 4] {
    let x = position.x;
    let y = position.y;

    let (w, h) = (dest_size.x, dest_size.y);

    let pivot = vec2(x + w / 2.0, y + h / 2.0);
    let m = pivot - vec2(w / 2.0, h / 2.0);

    let r = rotation;

    let p = [
        vec2(x, y) - pivot,
        vec2(x + w, y) - pivot,
        vec2(x + w, y + h) - pivot,
        vec2(x, y + h) - pivot,
    ];

    let p = [
        vec2(
            p[0].x * r.cos() - p[0].y * r.sin(),
            p[0].x * r.sin() + p[0].y * r.cos(),
        ) + m,
        vec2(
            p[1].x * r.cos() - p[1].y * r.sin(),
            p[1].x * r.sin() + p[1].y * r.cos(),
        ) + m,
        vec2(
            p[2].x * r.cos() - p[2].y * r.sin(),
            p[2].x * r.sin() + p[2].y * r.cos(),
        ) + m,
        vec2(
            p[3].x * r.cos() - p[3].y * r.sin(),
            p[3].x * r.sin() + p[3].y * r.cos(),
        ) + m,
    ];

    [
        SpriteVertex::new(
            vec3(p[0].x, p[0].y, position.z),
            vec2(0.0, 0.0),
            color,
        ),
        SpriteVertex::new(
            vec3(p[1].x, p[1].y, position.z),
            vec2(1.0, 0.0),
            color,
        ),
        SpriteVertex::new(
            vec3(p[2].x, p[2].y, position.z),
            vec2(1.0, 1.0),
            color,
        ),
        SpriteVertex::new(
            vec3(p[3].x, p[3].y, position.z),
            vec2(0.0, 1.0),
            color,
        ),
    ]
}
