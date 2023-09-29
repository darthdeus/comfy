use crate::*;

pub fn splat(v: f32) -> Vec2 {
    Vec2::splat(v)
}

pub fn isplat(v: i32) -> IVec2 {
    IVec2::splat(v)
}

pub fn usplat(v: u32) -> UVec2 {
    UVec2::splat(v)
}

pub fn simple_window(title: &str) -> egui::Window {
    egui::Window::new(title).resizable(false).collapsible(false)
}

pub trait Vec2EngineExtensions {
    fn as_world(&self) -> Position;
    fn as_world_size(&self) -> Size;
}

impl Vec2EngineExtensions for Vec2 {
    fn as_world(&self) -> Position {
        Position::world(self.x, self.y)
    }

    fn as_world_size(&self) -> Size {
        Size::world(self.x, self.y)
    }
}

pub fn get_fps() -> i32 {
    GLOBAL_STATE.borrow().fps
}

pub fn get_frame() -> u32 {
    GLOBAL_STATE.borrow().frame
}

pub fn inc_frame_num() {
    GLOBAL_STATE.borrow_mut().frame += 1;
}

pub fn sin_range(offset: f32, speed: f32, min: f32, max: f32) -> f32 {
    min + (max - min) *
        ((speed * (get_time() as f32 + offset)).sin() / 2.0 + 0.5)
}

pub fn clear_background(color: Color) {
    GLOBAL_STATE.borrow_mut().clear_color = color;
}

pub fn draw_quad(
    position: Vec2,
    size: Vec2,
    rotation: f32,
    color: Color,
    z_index: i32,
    texture: TextureHandle,
    scroll_offset: Vec2,
) {
    draw_sprite_ex(texture, position, color, z_index, DrawTextureParams {
        dest_size: Some(size.as_world_size()),
        scroll_offset,
        rotation,
        ..Default::default()
    });
}

pub fn draw_comfy(position: Vec2, tint: Color, z_index: i32, world_size: Vec2) {
    draw_sprite(
        texture_id("_builtin-comfy"),
        position,
        tint,
        z_index,
        world_size,
    );
}

/// Draws a sprite on the screen.
///
/// * `texture` - A handle to the texture to draw.
/// * `position` - World position where to draw.
/// * `tint` - The color tint to apply to the sprite.
/// * `z_index` - The z-index of the sprite. Higher values are drawn on top of lower values.
/// * `world_size` - The size of the world. Used for scaling.
pub fn draw_sprite(
    texture: TextureHandle,
    position: Vec2,
    tint: Color,
    z_index: i32,
    world_size: Vec2,
) {
    draw_sprite_rot(texture, position, tint, z_index, 0.0, world_size);
}


pub fn draw_sprite_ex(
    texture: TextureHandle,
    position: Vec2,
    tint: Color,
    z_index: i32,
    params: DrawTextureParams,
) {
    let _span = span!("draw_sprite_ex");

    let raw = RawDrawParams {
        dest_size: params.dest_size.map(|s| s.to_world()),
        source_rect: params.source_rect,
        rotation: params.rotation,
        flip_x: params.flip_x,
        flip_y: params.flip_y,
        pivot: params.pivot,
    };

    let size = Assets::image_size(texture).unwrap_or(UVec2::ONE);

    let vertices = rotated_rectangle(
        position.extend(z_index as f32),
        raw,
        size.x as f32,
        size.y as f32,
        tint,
        params.scroll_offset,
    );

    const QUAD_INDICES_U32: &[u32] = &[0, 2, 1, 0, 3, 2];

    let mesh = Mesh {
        vertices: SmallVec::from_slice(&vertices),
        indices: QUAD_INDICES_U32.into(),
        z_index,
        texture: Some(texture),
    };

    draw_mesh_ex(mesh, TextureParams {
        // TODO: shader
        shader: None,
        blend_mode: params.blend_mode,
    });
}

pub fn draw_rectangle_z_tex(
    position: Position,
    w: f32,
    h: f32,
    color: Color,
    z_index: i32,
    texture: Option<TextureHandle>,
    texture_params: TextureParams,
) {
    let (x, y) = position.to_world().tuple();

    let hw = w / 2.0;
    let hh = h / 2.0;

    let z = z_index as f32;

    #[rustfmt::skip]
    let vertices = [
        SpriteVertex::new(vec3(x - hw, y - hh, z), vec2(0.0, 0.0), color),
        SpriteVertex::new(vec3(x + hw, y - hh, z), vec2(1.0, 0.0), color),
        SpriteVertex::new(vec3(x + hw, y + hh, z), vec2(1.0, 1.0), color),
        SpriteVertex::new(vec3(x - hw, y + hh, z), vec2(0.0, 1.0), color),
    ];
    let indices = [0, 1, 2, 0, 2, 3];

    draw_mesh_ex(
        Mesh {
            vertices: SmallVec::from_slice(&vertices),
            indices: indices.into(),
            z_index,
            texture,
        },
        texture_params,
    );
}

pub fn draw_rect(center: Vec2, size: Vec2, color: Color, z_index: i32) {
    let _span = span!("draw_rect");
    draw_quad(center, size, 0.0, color, z_index, texture_id("1px"), Vec2::ZERO);
}

pub fn draw_rect_rot(
    center: Vec2,
    size: Vec2,
    rotation: f32,
    color: Color,
    z_index: i32,
) {
    let _span = span!("draw_rect_outline_rot");

    draw_quad(
        center,
        size,
        rotation,
        color,
        z_index,
        texture_id("1px"),
        Vec2::ZERO,
    );
}

pub fn draw_rect_outline(
    center: Vec2,
    size: Vec2,
    thickness: f32,
    color: Color,
    z_index: i32,
) {
    let _span = span!("draw_rect_outline");

    let (x, y) = center.tuple();
    let w = size.x;
    let h = size.y;

    let hw = w / 2.0;
    let hh = h / 2.0;

    let x = x - hw;
    let y = y - hh;

    let z = z_index as f32;

    // let t = thickness / 2.;
    // #[rustfmt::skip]
    // let vertices = vec![
    //     SpriteVertex::new(vec3(x    , y    , z), vec2(0.0, 1.0), color),
    //     SpriteVertex::new(vec3(x + w, y    , z), vec2(1.0, 0.0), color),
    //     SpriteVertex::new(vec3(x + w, y + h, z), vec2(1.0, 1.0), color),
    //     SpriteVertex::new(vec3(x    , y + h, z), vec2(0.0, 0.0), color),
    //     //inner rectangle
    //     SpriteVertex::new(vec3(x + t    , y + t    , z), vec2(0.0, 0.0), color),
    //     SpriteVertex::new(vec3(x + w - t, y + t    , z), vec2(0.0, 0.0), color),
    //     SpriteVertex::new(vec3(x + w - t, y + h - t, z), vec2(0.0, 0.0), color),
    //     SpriteVertex::new(vec3(x + t    , y + h - t, z), vec2(0.0, 0.0), color),
    // ];
    //
    // let indices: Vec<u32> = vec![
    //     0, 1, 4, 1, 4, 5, 1, 5, 6, 1, 2, 6, 3, 7, 2, 2, 7, 6, 0, 4, 3, 3, 4, 7,
    // ];

    let mut vertices = Vec::with_capacity(6 * 4);
    let mut indices = Vec::with_capacity(6 * 6);

    create_line_strip(
        &[
            vec2(x, y),
            vec2(x, y + h),
            vec2(x + w, y + h),
            vec2(x + w, y),
            vec2(x, y),
        ],
        thickness,
        &mut vertices,
        &mut indices,
    );

    let vertices = vertices
        .into_iter()
        .map(|v| SpriteVertex::new(v.extend(z), Vec2::ZERO, color))
        .collect_vec();

    draw_mesh(Mesh {
        vertices: vertices.into(),
        indices: indices.into(),
        z_index,
        texture: None,
    });
}

pub fn draw_labeled_rect_corners(
    label: &str,
    center: Vec2,
    size: Vec2,
    thickness: f32,
    corner_size: f32,
    color: Color,
    z_index: i32,
) {
    draw_text_ex(
        label,
        center + vec2(-size.x, size.y) / 2.0,
        TextAlign::BottomLeft,
        TextParams {
            font: egui::FontId::new(
                12.0 / egui_scale_factor(),
                egui::FontFamily::Proportional,
            ),
            rotation: 0.0,
            color: color.lighten(0.3),
        },
    );

    draw_rect_corners(center, size, thickness, corner_size, color, z_index);
}

pub fn labeled_hover_aabb(
    label: &str,
    aabb: &AABB,
    color: Color,
    z_index: i32,
) {
    let hover = aabb.contains(mouse_world());
    let color = if hover { color.lighten(0.15) } else { color.darken(0.1) };

    labeled_aabb(label, aabb, color, z_index);
}

pub fn labeled_aabb(label: &str, aabb: &AABB, color: Color, z_index: i32) {
    draw_labeled_rect_corners(
        label,
        aabb.center(),
        aabb.size(),
        4.0 * px(),
        1.0,
        color,
        z_index,
    );
}

pub fn draw_rect_corners(
    center: Vec2,
    size: Vec2,
    thickness: f32,
    corner_size: f32,
    color: Color,
    z_index: i32,
) {
    let (x, y) = center.tuple();
    let w = size.x;
    let h = size.y;

    let hw = w / 2.0;
    let hh = h / 2.0;

    let x = x - hw;
    let y = y - hh;

    let c = corner_size;
    let z = z_index as f32;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // bottom left
    create_line_strip(
        &[vec2(x, y + c), vec2(x, y), vec2(x + c, y)],
        thickness,
        &mut vertices,
        &mut indices,
    );

    // top right
    create_line_strip(
        &[vec2(x + w - c, y + h), vec2(x + w, y + h), vec2(x + w, y + h - c)],
        thickness,
        &mut vertices,
        &mut indices,
    );

    // bottom right
    create_line_strip(
        &[vec2(x + w - c, y), vec2(x + w, y), vec2(x + w, y + c)],
        thickness,
        &mut vertices,
        &mut indices,
    );

    // top left
    create_line_strip(
        &[vec2(x + c, y + h), vec2(x, y + h), vec2(x, y + h - c)],
        thickness,
        &mut vertices,
        &mut indices,
    );

    let vertices = vertices
        .into_iter()
        .map(|v| SpriteVertex::new(v.extend(z), Vec2::ZERO, color))
        .collect_vec();

    draw_mesh(Mesh {
        vertices: vertices.into(),
        indices: indices.into(),
        z_index,
        texture: None,
    });
}

pub fn create_line_strip(
    points: &[Vec2],
    thickness: f32,
    vertices: &mut Vec<Vec2>,
    indices: &mut Vec<u32>,
) {
    if points.len() < 2 {
        panic!("Not enough points to create a line strip!");
    }

    let half_thickness = thickness / 4.0;
    let idx_offset = vertices.len() as u32;

    for i in 0..(points.len() - 1) {
        let p0 = points[i];
        let p1 = points[i + 1];

        let direction = (p1 - p0).normalize_or_right();
        let normal = vec2(-direction.y, direction.x);

        vertices.push(p0 - normal * half_thickness);
        vertices.push(p0 + normal * half_thickness);
        vertices.push(p1 - normal * half_thickness);
        vertices.push(p1 + normal * half_thickness);

        let index_base = idx_offset + i as u32 * 4;

        indices.push(index_base);
        indices.push(index_base + 1);
        indices.push(index_base + 2);

        indices.push(index_base + 2);
        indices.push(index_base + 1);
        indices.push(index_base + 3);
    }
}

pub fn rotated_rectangle(
    position: Vec3,
    params: RawDrawParams,
    tex_width: f32,
    tex_height: f32,
    color: Color,
    scroll_offset: Vec2,
) -> [SpriteVertex; 4] {
    let x = position.x;
    let y = position.y;

    let dims = params
        .source_rect
        .map(|rect| {
            IRect {
                size: rect.size,
                offset: ivec2(
                    rect.offset.x,
                    tex_height as i32 - rect.offset.y - rect.size.y,
                ),
            }
        })
        .unwrap_or(IRect::new(
            ivec2(0, 0),
            ivec2(tex_width as i32, tex_height as i32),
        ));

    let sx = dims.offset.x as f32;
    let sy = dims.offset.y as f32;
    let sw = dims.size.x as f32;
    let sh = dims.size.y as f32;

    let (mut w, mut h) = match params.dest_size {
        Some(dst) => (dst.x, dst.y),
        _ => (1.0, 1.0),
    };

    if params.flip_x {
        w = -w;
    }
    if params.flip_y {
        h = -h;
    }

    let pivot = params.pivot.unwrap_or(vec2(x + w / 2.0, y + h / 2.0));
    let m = pivot - vec2(w / 2.0, h / 2.0);

    let r = params.rotation;

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
            vec2(sx / tex_width, sy / tex_height) + scroll_offset,
            color,
        ),
        SpriteVertex::new(
            vec3(p[1].x, p[1].y, position.z),
            vec2((sx + sw) / tex_width, sy / tex_height) + scroll_offset,
            color,
        ),
        SpriteVertex::new(
            vec3(p[2].x, p[2].y, position.z),
            vec2((sx + sw) / tex_width, (sy + sh) / tex_height) + scroll_offset,
            color,
        ),
        SpriteVertex::new(
            vec3(p[3].x, p[3].y, position.z),
            vec2(sx / tex_width, (sy + sh) / tex_height) + scroll_offset,
            color,
        ),
    ]
}

pub fn draw_rect_outline_rot(
    center: Vec2,
    size: Vec2,
    rotation: f32,
    thickness: f32,
    color: Color,
    z_index: i32,
) {
    let _span = span!("draw_rect_outline_rot");

    let (x, y) = center.tuple();
    let t = thickness / 2.;
    let w = size.x;
    let h = size.y;

    let hw = w / 2.0;
    let hh = h / 2.0;

    let x = x - hw;
    let y = y - hh;

    let z = z_index as f32;

    let pivot = vec2(x + w / 2.0, y + h / 2.0);

    #[rustfmt::skip]
    let mut vertices = [
        SpriteVertex::new(vec3(x    , y    , z), vec2(0.0, 1.0), color),
        SpriteVertex::new(vec3(x + w, y    , z), vec2(1.0, 0.0), color),
        SpriteVertex::new(vec3(x + w, y + h, z), vec2(1.0, 1.0), color),
        SpriteVertex::new(vec3(x    , y + h, z), vec2(0.0, 0.0), color),
        //inner rectangle
        SpriteVertex::new(vec3(x + t    , y + t    , z), vec2(0.0, 0.0), color),
        SpriteVertex::new(vec3(x + w - t, y + t    , z), vec2(0.0, 0.0), color),
        SpriteVertex::new(vec3(x + w - t, y + h - t, z), vec2(0.0, 0.0), color),
        SpriteVertex::new(vec3(x + t    , y + h - t, z), vec2(0.0, 0.0), color),
    ];

    // Apply rotation to points
    for p in &mut vertices {
        let px = p.position[0];
        let py = p.position[1];
        let pz = p.position[2];

        let new_px = (px - pivot.x) * rotation.cos() -
            (py - pivot.y) * rotation.sin() +
            pivot.x;

        let new_py = (px - pivot.x) * rotation.sin() +
            (py - pivot.y) * rotation.cos() +
            pivot.y;

        p.position = [new_px, new_py, pz];
    }

    let indices: Vec<u32> = vec![
        0, 1, 4, 1, 4, 5, 1, 5, 6, 1, 2, 6, 3, 7, 2, 2, 7, 6, 0, 4, 3, 3, 4, 7,
    ];

    draw_mesh(Mesh {
        vertices: SmallVec::from_slice(&vertices),
        indices: indices.into(),
        z_index,
        texture: None,
    });
}

pub fn draw_circle(center: Vec2, r: f32, color: Color, z_index: i32) {
    // let _span = span!("circle");
    draw_poly_z(center, 40, r, 0.0, color, z_index, TextureParams {
        blend_mode: BlendMode::Alpha,
        ..Default::default()
    });
}

pub fn draw_circle_outline(
    center: Vec2,
    radius: f32,
    thickness: f32,
    color: Color,
    z_index: i32,
) {
    let inner_radius = radius - thickness / 2.0;
    let outer_radius = radius + thickness / 2.0;

    let mut vertices = vec![];
    let mut indices = vec![];

    let mut prev_inner_point: Option<Vec2> = None;
    let mut prev_outer_point: Option<Vec2> = None;

    let step_size = 0.1;
    let steps = (2.0 * PI / step_size).round() as i32;

    for i in 0..=steps {
        let angle = i as f32 * step_size;
        let cos = angle.cos();
        let sin = angle.sin();

        let inner_point = Vec2::new(
            center.x + inner_radius * cos,
            center.y + inner_radius * sin,
        );
        let outer_point = Vec2::new(
            center.x + outer_radius * cos,
            center.y + outer_radius * sin,
        );

        if let (Some(prev_inner), Some(prev_outer)) =
            (prev_inner_point, prev_outer_point)
        {
            // Create two triangles
            let z = z_index as f32;

            vertices.push(SpriteVertex::new(
                vec3(prev_inner.x, prev_inner.y, z),
                vec2(0.0, 0.0),
                color,
            ));
            vertices.push(SpriteVertex::new(
                vec3(inner_point.x, inner_point.y, z),
                vec2(1.0, 0.0),
                color,
            ));
            vertices.push(SpriteVertex::new(
                vec3(prev_outer.x, prev_outer.y, z),
                vec2(0.0, 1.0),
                color,
            ));
            vertices.push(SpriteVertex::new(
                vec3(outer_point.x, outer_point.y, z),
                vec2(1.0, 1.0),
                color,
            ));

            let start_index = vertices.len() as u32 - 4;

            indices.extend_from_slice(&[
                start_index,
                start_index + 1,
                start_index + 2,
                start_index + 1,
                start_index + 2,
                start_index + 3,
            ]);
        }

        prev_inner_point = Some(inner_point);
        prev_outer_point = Some(outer_point);
    }

    draw_mesh(Mesh {
        vertices: vertices.into(),
        indices: indices.into(),
        z_index,
        texture: None,
    })
}

pub fn draw_circle_z(
    center: Vec2,
    r: f32,
    color: Color,
    z_index: i32,
    texture_params: TextureParams,
) {
    draw_poly_z(center, 40, r, 0.0, color, z_index, texture_params);
}

pub fn draw_line(
    p1: Vec2,
    p2: Vec2,
    thickness: f32,
    color: Color,
    z_index: i32,
) {
    draw_line_tex(p1, p2, thickness, z_index, color, None);
}

pub fn draw_ray(
    pos: Vec2,
    dir: Vec2,
    thickness: f32,
    color: Color,
    z_index: i32,
) {
    draw_line(pos, pos + dir, thickness, color, z_index);
}

pub fn draw_line_tex_y_uv_flex(
    p1: Position,
    p2: Position,
    start_thickness: f32,
    end_thickness: f32,
    color: Color,
    texture: Option<TextureHandle>,
    uv_offset: f32,
    uv_size: f32,
    z_index: i32,
    texture_params: TextureParams,
) {
    let (x1, y1) = p1.to_world().tuple();
    let (x2, y2) = p2.to_world().tuple();

    let dx = x2 - x1;
    let dy = y2 - y1;

    let nx = -dy;
    let ny = dx;

    let tlen = (nx * nx + ny * ny).sqrt();
    if tlen < std::f32::EPSILON {
        return;
    }

    let nxn = nx / tlen;
    let nyn = ny / tlen;

    let tx1 = nxn * start_thickness * 0.5;
    let ty1 = nyn * start_thickness * 0.5;

    let tx2 = nxn * end_thickness * 0.5;
    let ty2 = nyn * end_thickness * 0.5;

    let z = z_index as f32;

    // let wrapped_y_uv_start = uv_offset % 1.0;
    // let wrapped_y_uv_end = (uv_offset + uv_size) % 1.0;
    //

    let start = uv_offset % 1.0;
    let end = start + uv_size;

    // let start = wrapped_y_uv_start;
    // let end = wrapped_y_uv_end;

    // let y_uv_start = y_uv.start % 1.0;
    // let y_uv_end = y_uv.end % 1.0;

    // const EPSILON: f32 = 1e-6;
    //
    // let y_uv_start = y_uv.start % 1.0;
    // let mut y_uv_end = y_uv.end % 1.0;
    //
    // if y_uv_end.abs() < EPSILON {
    //     y_uv_end = 1.0;
    // }

    // let vertices = vec![
    //     SpriteVertex::new(
    //         vec3(x1 + tx1, y1 + ty1, z),
    //         vec2(0.0, y_uv_start),
    //         color,
    //     ),
    //     SpriteVertex::new(
    //         vec3(x1 - tx1, y1 - ty1, z),
    //         vec2(0.0, y_uv_end),
    //         color,
    //     ),
    //     SpriteVertex::new(
    //         vec3(x2 + tx2, y2 + ty2, z),
    //         vec2(1.0, y_uv_start),
    //         color,
    //     ),
    //     SpriteVertex::new(
    //         vec3(x2 - tx2, y2 - ty2, z),
    //         vec2(1.0, y_uv_end),
    //         color,
    //     ),
    // ];

    let top_left = vec3(x1 + tx1, y1 + ty1, z);
    let bottom_left = vec3(x1 - tx1, y1 - ty1, z);
    let top_right = vec3(x2 + tx2, y2 + ty2, z);
    let bottom_right = vec3(x2 - tx2, y2 - ty2, z);

    let vertices = [
        SpriteVertex::new(top_left, vec2(0.0, start), color),
        SpriteVertex::new(bottom_left, vec2(1.0, start), color),
        SpriteVertex::new(top_right, vec2(0.0, end), color),
        SpriteVertex::new(bottom_right, vec2(1.0, end), color),
    ];

    let indices = [0, 1, 2, 2, 1, 3];

    // println!("y_uv_start: {}, y_uv_end: {}", y_uv_start, y_uv_end);

    draw_mesh_ex(
        Mesh {
            vertices: SmallVec::from_slice(&vertices),
            indices: indices.into(),
            z_index,
            texture,
        },
        texture_params,
    )
}

pub fn draw_line_tex(
    p1: Vec2,
    p2: Vec2,
    thickness: f32,
    z_index: i32,
    color: Color,
    texture: Option<TextureHandle>,
) {
    let (x1, y1) = p1.tuple();
    let (x2, y2) = p2.tuple();

    let dx = x2 - x1;
    let dy = y2 - y1;

    // https://stackoverflow.com/questions/1243614/how-do-i-calculate-the-normal-vector-of-a-line-segment

    let nx = -dy;
    let ny = dx;

    let tlen = (nx * nx + ny * ny).sqrt() / (thickness * 0.5);
    if tlen < std::f32::EPSILON {
        return;
    }
    let tx = nx / tlen;
    let ty = ny / tlen;

    // 0 0      1 0
    //
    // 0 1      1 1

    let z = z_index as f32;

    let vertices = [
        SpriteVertex::new(vec3(x1 + tx, y1 + ty, z), vec2(0.0, 0.0), color),
        SpriteVertex::new(vec3(x1 - tx, y1 - ty, z), vec2(1.0, 0.0), color),
        SpriteVertex::new(vec3(x2 + tx, y2 + ty, z), vec2(0.0, 1.0), color),
        SpriteVertex::new(vec3(x2 - tx, y2 - ty, z), vec2(1.0, 1.0), color),
    ];

    // let vertices = vec![
    //     SpriteVertex::new(vec2(x1 + tx, y1 + ty), vec2(0.0, 0.0), color),
    //     SpriteVertex::new(vec2(x1 - tx, y1 - ty), vec2(1.0, 0.0), color),
    //     SpriteVertex::new(vec2(x2 + tx, y2 + ty), vec2(1.0, 1.0), color),
    //     SpriteVertex::new(vec2(x2 - tx, y2 - ty), vec2(0.0, 1.0), color),
    // ];

    let indices = [0, 1, 2, 2, 1, 3];

    draw_mesh(Mesh {
        vertices: SmallVec::from_slice(&vertices),
        indices: indices.into(),
        z_index,
        texture,
    })
}

pub fn draw_poly_z(
    position: Vec2,
    sides: u8,
    radius: f32,
    rotation: f32,
    color: Color,
    z_index: i32,
    texture_params: TextureParams,
) {
    let (x, y) = position.tuple();
    let z = z_index as f32;

    let mut vertices = Vec::<SpriteVertex>::with_capacity(sides as usize + 2);
    let mut indices = Vec::<u32>::with_capacity(sides as usize * 3);

    let rot = rotation.to_radians();
    vertices.push(SpriteVertex::new(vec3(x, y, z), vec2(0.0, 0.0), color));

    for i in 0..sides + 1 {
        let rx =
            (i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).cos();
        let ry =
            (i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).sin();

        let vertex = SpriteVertex::new(
            vec3(x + radius * rx, y + radius * ry, z),
            vec2(rx, ry),
            color,
        );

        vertices.push(vertex);

        if i != sides {
            indices.extend_from_slice(&[0, i as u32 + 1, i as u32 + 2]);
        }
    }

    draw_mesh_ex(
        Mesh {
            vertices: vertices.into(),
            indices: indices.into(),
            z_index,
            ..Default::default()
        },
        texture_params,
    );
}

pub fn draw_arc(
    position: Vec2,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    color: Color,
    z_index: i32,
) {
    let (x, y) = position.tuple();
    let z = z_index as f32;
    let segments = 40;

    let mut vertices =
        Vec::<SpriteVertex>::with_capacity(segments as usize + 2);
    let mut indices = Vec::<u32>::with_capacity(segments as usize * 3);

    vertices.push(SpriteVertex::new(vec3(x, y, z), vec2(0.0, 0.0), color));

    // if end_angle < 0.0 {
    //     end_angle = 2.0 * PI + end_angle;
    // }

    for i in 0..segments + 1 {
        let angle = start_angle +
            (i as f32 / segments as f32 * (end_angle - start_angle));

        let rx = angle.cos();
        let ry = angle.sin();

        let vertex = SpriteVertex::new(
            vec3(x + radius * rx, y + radius * ry, z),
            vec2(rx, ry),
            color,
        );

        vertices.push(vertex);

        if i != segments {
            indices.extend_from_slice(&[0, i as u32 + 1, i as u32 + 2]);
        }
    }

    draw_mesh_ex(
        Mesh {
            vertices: vertices.into(),
            indices: indices.into(),
            z_index,
            ..Default::default()
        },
        TextureParams::default(),
        // texture_params,
    );
}

pub fn draw_arc_outline(
    center: Vec2,
    radius: f32,
    thickness: f32,
    start_angle: f32,
    end_angle: f32,
    color: Color,
    z_index: i32,
) {
    let inner_radius = radius - thickness / 2.0;
    let outer_radius = radius + thickness / 2.0;

    let two_pi = 2.0 * PI;
    let start_angle = start_angle % two_pi;
    let mut end_angle = end_angle % two_pi;

    if end_angle < start_angle {
        end_angle += two_pi;
    }

    let mut vertices = vec![];
    let mut indices = vec![];

    let mut prev_inner_point: Option<Vec2> = None;
    let mut prev_outer_point: Option<Vec2> = None;

    let step_size = 0.1;
    let steps = ((end_angle - start_angle) / step_size).round() as i32;

    for i in 0..=steps {
        let angle = start_angle + i as f32 * step_size;
        let cos = angle.cos();
        let sin = angle.sin();

        let inner_point = Vec2::new(
            center.x + inner_radius * cos,
            center.y + inner_radius * sin,
        );
        let outer_point = Vec2::new(
            center.x + outer_radius * cos,
            center.y + outer_radius * sin,
        );

        if let (Some(prev_inner), Some(prev_outer)) =
            (prev_inner_point, prev_outer_point)
        {
            let z = z_index as f32;

            vertices.push(SpriteVertex::new(
                vec3(prev_inner.x, prev_inner.y, z),
                vec2(0.0, 0.0),
                color,
            ));
            vertices.push(SpriteVertex::new(
                vec3(inner_point.x, inner_point.y, z),
                vec2(1.0, 0.0),
                color,
            ));
            vertices.push(SpriteVertex::new(
                vec3(prev_outer.x, prev_outer.y, z),
                vec2(0.0, 1.0),
                color,
            ));
            vertices.push(SpriteVertex::new(
                vec3(outer_point.x, outer_point.y, z),
                vec2(1.0, 1.0),
                color,
            ));

            let start_index = vertices.len() as u32 - 4;

            indices.extend_from_slice(&[
                start_index,
                start_index + 1,
                start_index + 2,
                start_index + 1,
                start_index + 2,
                start_index + 3,
            ]);
        }

        prev_inner_point = Some(inner_point);
        prev_outer_point = Some(outer_point);
    }

    draw_mesh(Mesh {
        vertices: vertices.into(),
        indices: indices.into(),
        z_index,
        texture: None,
    })
}

pub fn draw_arc_wedge(
    center: Vec2,
    radius: f32,
    thickness: f32,
    start_angle: f32,
    end_angle: f32,
    color: Color,
    z_index: i32,
) {
    draw_arc_outline(
        center,
        radius,
        thickness,
        start_angle,
        end_angle,
        color,
        z_index,
    );

    let start_point = vec2(start_angle.cos(), start_angle.sin()) * radius;
    let end_point = vec2(end_angle.cos(), end_angle.sin()) * radius;

    draw_line(center, center + start_point, thickness, color, z_index);
    draw_line(center, center + end_point, thickness, color, z_index);
}

pub fn draw_wedge(
    center: Vec2,
    radius: f32,
    thickness: f32,
    start_angle: f32,
    end_angle: f32,
    color: Color,
    z_index: i32,
) {
    let start_point = vec2(start_angle.cos(), start_angle.sin()) * radius;
    let end_point = vec2(end_angle.cos(), end_angle.sin()) * radius;

    draw_line(center, center + start_point, thickness, color, z_index);
    draw_line(center, center + end_point, thickness, color, z_index);

    draw_line(
        center + start_point,
        center + end_point,
        thickness,
        color,
        z_index,
    );
}

pub fn draw_arrow(
    start: Vec2,
    end: Vec2,
    thickness: f32,
    color: Color,
    z_index: i32,
) {
    let dir = end - start;

    let angle = dir.angle();

    let len = 0.8;
    let spread = 0.15 * PI;

    // draw the arrow head
    draw_ray(
        end,
        -Vec2::from_angle(angle + spread) * len,
        thickness,
        color,
        z_index,
    );
    draw_ray(
        end,
        -Vec2::from_angle(angle - spread) * len,
        thickness,
        color,
        z_index,
    );

    draw_ray(start, dir, thickness, color, z_index);
}

pub fn draw_revs(position: Vec2, r: f32, rev: f32, color: Color, z_index: i32) {
    let rev_end_angle = PI / 4.0;

    let px = px();
    let offset = 3.0 * px;

    draw_arc_outline(
        position,
        r + offset,
        LINE_W * px,
        PI + rev_end_angle - rev,
        PI + rev_end_angle,
        color,
        z_index,
    );

    draw_arc_outline(
        position,
        r + offset,
        LINE_W * px,
        -rev_end_angle,
        -rev_end_angle + rev,
        color,
        z_index,
    );
}

pub fn draw_mesh(mesh: Mesh) {
    GLOBAL_STATE
        .borrow_mut()
        .mesh_queue
        .push(MeshDraw { mesh, texture_params: TextureParams::default() });
}

pub fn draw_mesh_ex(mesh: Mesh, texture_params: TextureParams) {
    GLOBAL_STATE
        .borrow_mut()
        .mesh_queue
        .push(MeshDraw { mesh, texture_params });
}

#[derive(Copy, Clone, Debug)]
pub struct DrawTextureParams {
    pub dest_size: Option<Size>,
    pub source_rect: Option<IRect>,
    pub scroll_offset: Vec2,
    pub rotation: f32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub pivot: Option<Vec2>,
    pub blend_mode: BlendMode,
}

impl Default for DrawTextureParams {
    fn default() -> DrawTextureParams {
        DrawTextureParams {
            dest_size: None,
            source_rect: None,
            scroll_offset: Vec2::ZERO,
            rotation: 0.,
            pivot: None,
            flip_x: false,
            flip_y: false,
            blend_mode: BlendMode::None,
        }
    }
}

impl DrawTextureParams {
    pub fn blend(blend_mode: BlendMode) -> DrawTextureParams {
        DrawTextureParams { blend_mode, ..Default::default() }
    }
}

pub fn draw_line_tex_y_uv(
    p1: Position,
    p2: Position,
    thickness: f32,
    color: Color,
    texture: Option<TextureHandle>,
    y_uv: Range<f32>,
    z_index: i32,
    texture_params: TextureParams,
) {
    let (x1, y1) = p1.to_world().tuple();
    let (x2, y2) = p2.to_world().tuple();

    let dx = x2 - x1;
    let dy = y2 - y1;

    // https://stackoverflow.com/questions/1243614/how-do-i-calculate-the-normal-vector-of-a-line-segment

    let nx = -dy;
    let ny = dx;

    let tlen = (nx * nx + ny * ny).sqrt() / (thickness * 0.5);
    if tlen < std::f32::EPSILON {
        return;
    }
    let tx = nx / tlen;
    let ty = ny / tlen;

    let z = z_index as f32;

    // 0 0      1 0
    //
    // 0 1      1 1
    let y_uv_start = y_uv.start % 1.0;
    let y_uv_end = y_uv.end % 1.0;

    let vertices = [
        SpriteVertex::new(
            vec3(x1 + tx, y1 + ty, z),
            vec2(0.0, y_uv_start),
            color,
        ),
        SpriteVertex::new(
            vec3(x1 - tx, y1 - ty, z),
            vec2(1.0, y_uv_start),
            color,
        ),
        SpriteVertex::new(
            vec3(x2 + tx, y2 + ty, z),
            vec2(0.0, y_uv_end),
            color,
        ),
        SpriteVertex::new(
            vec3(x2 - tx, y2 - ty, z),
            vec2(1.0, y_uv_end),
            color,
        ),
    ];

    // let vertices = vec![
    //     SpriteVertex::new(vec2(x1 + tx, y1 + ty), vec2(0.0, 0.0), color),
    //     SpriteVertex::new(vec2(x1 - tx, y1 - ty), vec2(1.0, 0.0), color),
    //     SpriteVertex::new(vec2(x2 + tx, y2 + ty), vec2(1.0, 1.0), color),
    //     SpriteVertex::new(vec2(x2 - tx, y2 - ty), vec2(0.0, 1.0), color),
    // ];

    let indices = [0, 1, 2, 2, 1, 3];

    draw_mesh_ex(
        Mesh {
            vertices: SmallVec::from_slice(&vertices),
            indices: indices.into(),
            z_index: 0,
            texture,
        },
        texture_params,
    )
}
