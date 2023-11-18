use crate::*;

pub const Z_DEBUG: i32 = 95;
const RAY_WIDTH: f32 = 0.5;

pub struct DebugMeta {
    pub category: &'static str,
}

impl DebugMeta {
    pub fn new(category: &'static str) -> Self {
        DebugMeta { category }
    }
}

pub struct DebugMark {
    pub pos: Position,
    pub color: Color,
    pub lifetime: f32,
}

pub struct DebugDraw {
    pub entities: Vec<Entity>,
    pub messages: Vec<String>,
    pub queue: Vec<DrawItem>,
}

pub enum DrawItem {
    Ray { origin: Vec2, dir: Vec2 },
    Line { a: Vec2, b: Vec2, width: f32, color: Color },
    Circle { center: Vec2, radius: f32, color: Color },
    Sprite { center: Vec2, texture: TextureHandle, rect: Option<Rect> },
}

impl DebugDraw {
    #[allow(dead_code)]
    fn update(&mut self) {
        for item in self.queue.drain(..) {
            match item {
                DrawItem::Line { a, b, width, color } => {
                    draw_line(a, b, width, color, Z_DEBUG);
                }
                DrawItem::Circle { center, radius, color } => {
                    draw_circle(center, radius, color, Z_DEBUG);
                }
                DrawItem::Sprite { center, texture, rect } => {
                    draw_sprite_ex(
                        texture,
                        center,
                        WHITE,
                        Z_DEBUG,
                        DrawTextureParams {
                            source_rect: rect.map(|r| {
                                IRect::new(
                                    r.top_left().as_ivec2(),
                                    r.size.as_ivec2(),
                                )
                            }),
                            ..Default::default()
                        },
                    );
                }
                DrawItem::Ray { origin, dir } => {
                    draw_line(
                        origin,
                        origin + dir,
                        RAY_WIDTH,
                        RED.alpha(0.7),
                        Z_DEBUG,
                    );
                }
            }
        }
    }


    pub fn line(&mut self, a: Vec2, b: Vec2) {
        self.queue.push(DrawItem::Line { a, b, width: 4.0, color: RED });
    }

    pub fn line_color(&mut self, a: Vec2, b: Vec2, color: Color) {
        self.queue.push(DrawItem::Line { a, b, width: 4.0, color });
    }

    pub fn sprite(&mut self, center: Vec2, texture: TextureHandle) {
        self.queue.push(DrawItem::Sprite { center, texture, rect: None });
    }

    pub fn sprite_rect(
        &mut self,
        center: Vec2,
        texture: TextureHandle,
        rect: Rect,
    ) {
        self.queue.push(DrawItem::Sprite { center, texture, rect: Some(rect) });
    }

    pub fn circle(&mut self, center: Vec2, radius: f32) {
        self.queue.push(DrawItem::Circle { center, radius, color: RED })
    }

    pub fn circle_color(&mut self, center: Vec2, radius: f32, color: Color) {
        self.queue.push(DrawItem::Circle { center, radius, color })
    }

    pub fn ray(&mut self, origin: Vec2, dir: Vec2) {
        self.line(origin, origin + dir);
    }
}

// TODO: replace & get rid of this
// pub fn render_colliders(c: &mut Context) {
//     let mut to_draw = vec![];
//
//     for (ch, collider) in c.physics.collider_set.iter() {
//         let entity = match Entity::from_bits(collider.user_data as u64) {
//             Some(entity) => entity,
//             None => {
//                 continue;
//             }
//         };
//
//         let rect = collider.compute_aabb();
//
//         if entity.has::<PhysicsDrawBall>(world) {
//             continue;
//         }
//
//         let color = if let Ok(color) = world.get::<&ColoredCollider>(entity) {
//             color.0
//         } else {
//             if !c.config.debug_collider_bounds {
//                 continue;
//             } else {
//                 match collider.shape_type() {
//                     ColliderDrawShape::Unknown => PINK,
//                     ColliderDrawShape::Cuboid => RED,
//                     ColliderDrawShape::Ball => PINK,
//                     ColliderDrawShape::ConvexPolygon => BLUE,
//                     ColliderDrawShape::Compound => GREEN,
//                     ColliderDrawShape::Trimesh => LIME,
//                     ColliderDrawShape::Polyline => BLUE,
//                     ColliderDrawShape::Capsule => BLUE,
//                 }
//             }
//         };
//
//         let iso = if let Ok(rbd_handle) = world.get::<&RigidBodyHandle>(entity)
//         {
//             if let Some(rbd) = c.physics.rigid_body_set.get(*rbd_handle) {
//                 *rbd.position() *
//                     *collider
//                         .position_wrt_parent()
//                         .unwrap_or(&Isometry::identity())
//             } else {
//                 *collider.position()
//             }
//         } else {
//             *collider.position()
//         };
//
//         if c.config.debug_labels {
//             if let Ok(label) = world.get::<&Label>(entity) {
//                 c.draw.text(
//                     Position::world(
//                         rect.mins.x * 16.0 - 64.0,
//                         rect.mins.y * 16.0 + 32.0,
//                     ),
//                     label.0.clone(),
//                     DrawTextParams {
//                         layer: layers::DEBUG_FRONT + 100,
//                         px_size: 8.0,
//                     },
//                 );
//             }
//         }
//
//         to_draw.push((entity, ch, iso, color));
//     }
//
//     for (entity, col_handle, iso, color) in to_draw.into_iter() {
//         if let Some(collider) = c.physics.collider_set.get(col_handle) {
//             render_collider(
//                 c.draw, world, entity, col_handle, iso, collider, color,
//             );
//         }
//     }
// }

// #[derive(Copy, Clone, Debug, PartialEq)]
// pub enum ColliderDrawShape {
//     Unknown = 0,
//     Cuboid,
//     Ball,
//     ConvexPolygon,
//     Compound,
//     Trimesh,
//     Polyline,
//     Capsule,
// }
//
// impl ColliderDrawShape {
//     pub fn from_num(num: u64) -> Option<ColliderDrawShape> {
//         match num {
//             0 => Some(ColliderDrawShape::Unknown),
//             1 => Some(ColliderDrawShape::Cuboid),
//             2 => Some(ColliderDrawShape::Ball),
//             3 => Some(ColliderDrawShape::ConvexPolygon),
//             4 => Some(ColliderDrawShape::Compound),
//             5 => Some(ColliderDrawShape::Trimesh),
//             6 => Some(ColliderDrawShape::Polyline),
//             7 => Some(ColliderDrawShape::Capsule),
//             _ => None,
//         }
//     }
// }
//
// #[test]
// fn collider_conversion() {
//     let shapes = [
//         ColliderDrawShape::Unknown,
//         ColliderDrawShape::Cuboid,
//         ColliderDrawShape::Ball,
//         ColliderDrawShape::ConvexPolygon,
//         ColliderDrawShape::Compound,
//         ColliderDrawShape::Trimesh,
//         ColliderDrawShape::Polyline,
//         ColliderDrawShape::Capsule,
//     ];
//
//     for shape in shapes.into_iter() {
//         assert_eq!(shape, ColliderDrawShape::from_num(shape as u64).unwrap());
//     }
// }

// #[repr(C)]
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct PartColUserData {
//     ship_entity: Entity,
//     part_col_handle: ColliderHandle,
// }
//
// impl PartColUserData {
//     pub fn make_bits(
//         ship_entity: Entity,
//         part_col_handle: ColliderHandle,
//     ) -> u128 {
//         PartColUserData { ship_entity, part_col_handle }.to_bits()
//     }
//
//     // pub fn make_col_bits(entity: Entity, color: Color, collider: &Collider) -> u128 {
//     //     UserData {
//     //         shape: collider.shape_type(),
//     //         entity,
//     //     }
//     //     .to_bits()
//     // }
//
//     pub fn to_bits(&self) -> u128 {
//         ((self.shape as u128) << 64) | self.entity.to_bits().get() as u128
//         // PackedUserData::new().to_bits()
//     }
//
//     pub fn from_bits(bits: u128) -> Option<Self> {
//         Entity::from_bits(bits as u64).and_then(|entity| {
//             let shape_bits = ((bits &
//                 ((0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFFu128 << 64)
//                     as u128)) >>
//                 64) as u64;
//
//             ColliderDrawShape::from_num(shape_bits)
//                 .map(|shape| PartColUserData { entity, color: RED, shape })
//         })
//     }
// }
//
// #[test]
// fn user_data_test() {
//     let data = PartColUserData {
//         shape: ColliderDrawShape::Ball,
//         color: RED,
//         entity: Entity::DANGLING,
//     };
//
//     let bits = data.to_bits();
//
//     assert_eq!(Some(data), PartColUserData::from_bits(bits));
// }

// struct PointBasedShape {
//     points: Vec<Vec2>,
//     origin: Vec2,
// }


// pub fn render_collider(
//     draw: &mut DrawCommands,
//     _world: &World,
//     _entity: Entity,
//     _col_handle: ColliderHandle,
//     iso: Isometry,
//     collider: &Collider,
//     color: Color,
// ) -> Option<()> {
//     let closed_line = DrawMode::LineClosed(8.0);
//     let layer = layers::LEVEL;
//
//     if let Some(compound) = collider.shape().as_compound() {
//         for (off, shape) in compound.shapes().iter() {
//             let iso = iso * off;
//
//             if let Some(polygon) = shape.as_convex_polygon() {
//                 draw.transformed_points(
//                     polygon.points(),
//                     &iso,
//                     layer,
//                     color,
//                     closed_line,
//                 );
//             }
//         }
//     }
//
//     if let Some(polygon) = collider.shape().as_convex_polygon() {
//         draw.transformed_points(
//             polygon.points(),
//             &iso,
//             layer,
//             color,
//             DrawMode::FilledPolygon,
//         );
//     }
//
//     if let Some(ball) = collider.shape().as_ball() {
//         draw.transformed_points(
//             &ball.to_polyline(16),
//             &iso,
//             layer,
//             color,
//             DrawMode::FilledPolygon,
//         );
//     }
//
//     if let Some(polyline) = collider.shape().as_polyline() {
//         draw.transformed_points(
//             polyline.vertices(),
//             &iso,
//             layer,
//             color,
//             closed_line,
//         );
//     }
//
//     if let Some(capsule) = collider.shape().as_capsule() {
//         let polyline = capsule.to_polyline(4);
//
//         draw.transformed_points(
//             &polyline,
//             &iso,
//             layer,
//             color,
//             DrawMode::FilledPolygon,
//         );
//     }
//
//     if let Some(cuboid) = collider.shape().as_cuboid() {
//         let rect = cuboid.compute_aabb(&iso);
//         draw.rect(
//             // Position::physics(iso.translation.x, iso.translation.y),
//             Position::world(rect.mins.x * 16.0, rect.mins.y * 16.0),
//             rect.extents().x * 16.0,
//             rect.extents().y * 16.0,
//             layer,
//             color,
//         )
//     }
//
//     Some(())
// }
