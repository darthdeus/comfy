use crate::*;

#[derive(Copy, Clone, Debug)]
pub struct QuadDraw {
    pub transform: Transform,
    pub texture: TextureHandle,
    pub z_index: i32,
    pub color: Color,
    pub blend_mode: BlendMode,
    pub source_rect: Option<IRect>,
    pub dest_size: Vec2,
    pub rotation_x: f32,
    pub flip_x: bool,
    pub flip_y: bool,
}

// // TODO: move this into quad
// pub fn draw_collider_p(
//     box_texture: TextureHandle,
//     collider: &Collider,
//     color: Color,
// ) {
//     let aabb: &Cuboid = collider.shape().as_cuboid().unwrap();
//
//     let w = aabb.half_extents.x * 2.0;
//     let h = aabb.half_extents.y * 2.0;
//
//     draw_texture_z_ex(
//         box_texture,
//         vec2(
//             collider.absolute_translation().x - 0.5 * w,
//             -collider.absolute_translation().y - 0.5 * h,
//         ),
//         color,
//         50, // TODO
//         DrawTextureParams {
//             dest_size: Some(Size::world(w, h)),
//             rotation: -collider.absolute_rotation(),
//             ..Default::default()
//         },
//     );
// }

// pub fn draw_aabb(aabb: Aabb, _thickness: f32, color: Color) {
//     let scale = 1.0;
//
//     draw_rectangle(
//         Position::world(aabb.mins.x * scale, -aabb.mins.y * scale),
//         aabb.extents().x * scale,
//         aabb.extents().y * scale,
//         color,
//     );
// }
