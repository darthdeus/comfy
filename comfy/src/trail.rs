use crate::*;

#[derive(Clone, Debug)]
pub struct Trail {
    pub positions: Vec<Vec2>,
    pub last_vertex_at: Vec2,

    pub z_index: i32,

    pub is_enabled: bool,

    pub trail_length: f32,
    pub width: f32,
    pub color_start: Color,
    pub color_end: Color,

    pub max_vertices: usize,
    pub fade_start_distance: f32,
    pub fade_end_distance: f32,
    pub width_curve: Option<Curve>,
    pub color_curve: Option<ColorCurve>,
    pub texture: Option<TextureHandle>,
    pub blend_mode: BlendMode,
}

impl Trail {
    pub fn new(
        width: f32,
        trail_length: f32,
        z_index: i32,
        color_start: Color,
        color_end: Color,
        max_vertices: usize,
        fade_start_distance: f32,
        fade_end_distance: f32,
        // width_curve: Option<Curve>,
        color_curve: Option<ColorCurve>,
        texture: Option<TextureHandle>,
        blend_mode: BlendMode,
    ) -> Self {
        Self {
            positions: vec![],
            last_vertex_at: Vec2::ZERO,

            z_index,

            is_enabled: true,

            trail_length,
            width,
            color_start,
            color_end,

            max_vertices,
            fade_start_distance,
            fade_end_distance,
            width_curve: None,
            color_curve,
            texture,
            blend_mode,
        }
    }

    pub fn simple(
        width: f32,
        trail_length: f32,
        z_index: i32,
        color_start: Color,
        color_end: Color,
    ) -> Self {
        Self::new(
            width,
            trail_length,
            z_index,
            color_start,
            color_end,
            100,
            0.0,
            0.0,
            None,
            None,
            BlendMode::Additive,
        )
    }

    pub fn update(&mut self, position: Vec2, _delta: f32) {
        if self.is_enabled {
            let distance = (position - self.last_vertex_at).length();

            let min_vertex_distance =
                self.trail_length / self.max_vertices as f32;

            if self.positions.is_empty() {
                self.positions.push(position);
                self.last_vertex_at = position;
            } else if distance > min_vertex_distance {
                // The number of interpolation steps is the distance divided by min_vertex_distance, rounded up
                let num_steps =
                    (distance / min_vertex_distance).ceil() as usize;
                for i in 1..=num_steps {
                    // Compute the interpolated position
                    let t = i as f32 / num_steps as f32;
                    let interpolated_position =
                        self.last_vertex_at * (1.0 - t) + position * t;
                    self.positions.push(interpolated_position);
                }

                let mut total_distance = self.total_distance();

                while self.positions.len() > 2 &&
                    total_distance > self.trail_length
                {
                    self.positions.remove(0);
                    total_distance = self.total_distance();
                }

                self.last_vertex_at = position;

                // self.positions.push(position);
                //
                // let mut total_distance = self.total_distance();
                //
                // while self.positions.len() > 2 &&
                //     total_distance > self.trail_length
                // {
                //     self.positions.remove(0);
                //     total_distance = self.total_distance();
                // }
                //
                // self.last_vertex_at = position;
            }

            // self.positions.push(position);
        } else if let Some(first_position) = self.positions.first() {
            if (*first_position - position).length() > self.trail_length {
                self.positions.remove(0);
            }
        }
    }

    fn total_distance(&self) -> f32 {
        self.positions.windows(2).map(|w| (w[0] - w[1]).length()).sum()
    }

    pub fn draw_mesh(&self) {
        if self.positions.len() <= 1 {
            return;
        }

        let tex = self.texture.unwrap_or(texture_id("1px"));
        // let tex = texture_id("1px");

        // let mut trail_length = 0.0;

        let mut vertices = vec![];

        for (i, (a, b)) in self.positions.iter().tuple_windows().enumerate() {
            let n = self.positions.len() as f32;
            // let step = 1.0 / n;
            let pct = i as f32 / n;

            let off = 2.0 * (get_unpaused_time() as f32 % 1.0);

            let width_pct_a = self
                .width_curve
                .as_ref()
                .map(|curve| curve.eval(pct))
                .unwrap_or(pct);

            // let width_pct_b = self
            //     .width_curve
            //     .as_ref()
            //     .map(|curve| curve.eval(pct + step))
            //     .unwrap_or(pct + step);

            let color = self
                .color_curve
                .as_ref()
                .map(|curve| curve.eval(pct))
                .unwrap_or(self.color_start.lerp(self.color_end, 1.0 - pct));

            // If the trail is not enabled, fade the color out based on the distance
            let color = if self.is_enabled {
                color
            } else {
                let distance_pct = (i as f32 / n).powi(2);
                color.darken(distance_pct)
            };

            let p1 = Position::world(a.x, a.y);
            let p2 = Position::world(b.x, b.y);

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


            let start_thickness = self.width * width_pct_a;
            // let end_thickness = self.width * width_pct_b;

            let tx1 = nxn * start_thickness * 0.5;
            let ty1 = nyn * start_thickness * 0.5;

            // let tx2 = nxn * end_thickness * 0.5;
            // let ty2 = nyn * end_thickness * 0.5;

            let z = self.z_index as f32;

            // let wrapped_y_uv_start = uv_offset % 1.0;
            // let wrapped_y_uv_end = (uv_offset + uv_size) % 1.0;

            let start = off + pct;
            // let uv_size = step;

            // let start = uv_offset;
            // let end = start + uv_size;

            let top_left = vec3(x1 + tx1, y1 + ty1, z);
            let bottom_left = vec3(x1 - tx1, y1 - ty1, z);

            vertices.push(SpriteVertex::new(top_left, vec2(0.0, start), color));
            vertices.push(SpriteVertex::new(
                bottom_left,
                vec2(1.0, start),
                color,
            ));

            // draw_line_tex_y_uv_flex(
            //     // 1.0,
            //     // 1.0,
            //     // color * alpha,
            //     color,
            //     Some(tex),
            //     self.z_index,
            //     TextureParams {
            //         blend_mode: self.blend_mode,
            //         ..Default::default()
            //     },
            // );
        }

        let indices = Self::generate_triangle_list_indices(vertices.len());

        draw_mesh_ex(
            Mesh {
                // TODO: might want the average instead
                origin: self
                    .positions
                    .last()
                    .copied()
                    .unwrap_or_default()
                    .extend(self.z_index as f32),
                vertices: vertices.into(),
                indices: indices.into(),
                z_index: self.z_index,
                texture: Some(tex),
            },
            TextureParams { blend_mode: BlendMode::Additive },
        );
    }

    fn generate_triangle_list_indices(n: usize) -> Vec<u32> {
        let mut indices = Vec::with_capacity(2 * (n - 2));
        let mut is_even = true;

        for i in 1..(n as u32 - 1) {
            if is_even {
                indices.push(i - 1);
                indices.push(i);
                indices.push(i + 1);
            } else {
                indices.push(i + 1);
                indices.push(i);
                indices.push(i - 1);
            }

            is_even = !is_even;
        }

        indices
    }

    // pub fn draw(&self) {
    //     if self.positions.len() <= 1 {
    //         return;
    //     }
    //
    //     let tex = self.texture.unwrap_or(texture_id("trail"));
    //     // let tex = texture_id("1px");
    //
    //     let mut trail_length = 0.0;
    //
    //     // let mesh = vec![];
    //
    //     for (i, (a, b)) in self.positions.iter().tuple_windows().enumerate() {
    //         let n = self.positions.len() as f32;
    //         let step = 1.0 / n;
    //         let pct = i as f32 / n;
    //
    //         let off = 2.0 * get_unpaused_time() as f32;
    //
    //         let width_pct_a = self
    //             .width_curve
    //             .as_ref()
    //             .map(|curve| curve.eval(pct))
    //             .unwrap_or(pct);
    //
    //         let width_pct_b = self
    //             .width_curve
    //             .as_ref()
    //             .map(|curve| curve.eval(pct + step))
    //             .unwrap_or(pct + step);
    //
    //         // let width_pct_a = pct;
    //         // let width_pct_b = pct + step;
    //
    //         let color = self
    //             .color_curve
    //             .as_ref()
    //             .map(|curve| curve.eval(pct))
    //             .unwrap_or(self.color_start.lerp(self.color_end, 1.0 - pct));
    //
    //         trail_length += (*b - *a).length();
    //
    //         let fade_start = trail_length - self.fade_start_distance;
    //         let fade_end = trail_length - self.fade_end_distance;
    //
    //         let alpha = if trail_length < self.fade_start_distance {
    //             1.0
    //         } else if trail_length > self.fade_end_distance {
    //             0.0
    //         } else {
    //             1.0 - (fade_start / (fade_end - fade_start))
    //         };
    //
    //         draw_line_tex_y_uv_flex(
    //             Position::world(a.x, a.y),
    //             Position::world(b.x, b.y),
    //             self.width * width_pct_a,
    //             self.width * width_pct_b,
    //             // 1.0,
    //             // 1.0,
    //             // color * alpha,
    //             color,
    //             Some(tex),
    //             off + pct,
    //             step,
    //             self.z_index,
    //             TextureParams {
    //                 blend_mode: self.blend_mode,
    //                 ..Default::default()
    //             },
    //         );
    //     }
    // }

    // pub fn draw(&self) {
    //     if self.positions.len() <= 1 {
    //         return;
    //     }
    //
    //     // let tex = self.texture.unwrap_or(texture_id("trail"));
    //     let tex = texture_id("1px");
    //
    //     let mut trail_length = 0.0;
    //
    //     for (i, (a, b)) in self.positions.iter().tuple_windows().enumerate() {
    //         let n = self.positions.len() as f32;
    //         let step = 1.0 / n;
    //         let pct = i as f32 / n;
    //
    //         let off = 2.0 * get_unpaused_time() as f32;
    //
    //         let width_pct = self
    //             .width_curve
    //             .as_ref()
    //             .map(|curve| curve.eval(pct))
    //             .unwrap_or(pct);
    //
    //         let width_pct_a = self
    //             .width_curve
    //             .as_ref()
    //             .map(|curve| curve.eval(pct))
    //             .unwrap_or(pct);
    //         let width_pct_b = self
    //             .width_curve
    //             .as_ref()
    //             .map(|curve| curve.eval(pct + step))
    //             .unwrap_or(pct + step);
    //
    //         let color = self
    //             .color_curve
    //             .as_ref()
    //             .map(|curve| curve.eval(pct))
    //             .unwrap_or(self.color_start.lerp(self.color_end, 1.0 - pct));
    //
    //         trail_length += (*b - *a).length();
    //
    //         let fade_start = trail_length - self.fade_start_distance;
    //         let fade_end = trail_length - self.fade_end_distance;
    //
    //         let alpha = if trail_length < self.fade_start_distance {
    //             1.0
    //         } else if trail_length > self.fade_end_distance {
    //             0.0
    //         } else {
    //             1.0 - (fade_start / (fade_end - fade_start))
    //         };
    //
    //         draw_line_tex_y_uv_flex(
    //             Position::world(a.x, a.y),
    //             Position::world(b.x, b.y),
    //             self.width * width_pct_a,
    //             self.width * width_pct_b,
    //             color * alpha,
    //             Some(tex),
    //             (off + pct)..(off + pct + step),
    //             self.z_index,
    //             TextureParams {
    //                 blend_mode: self.blend_mode,
    //                 ..Default::default()
    //             },
    //         );
    //
    //         // draw_line_tex_y_uv(
    //         //     Position::world(a.x, a.y),
    //         //     Position::world(b.x, b.y),
    //         //     self.width * width_pct,
    //         //     color * alpha,
    //         //     Some(tex),
    //         //     (off + pct)..(off + pct + step),
    //         //     self.z_index,
    //         //     TextureParams {
    //         //         blend_mode: self.blend_mode,
    //         //         ..Default::default()
    //         //     },
    //         // );
    //     }
    // }
}

#[derive(Clone, Debug)]
pub struct Curve {
    pub points: Vec<(f32, f32)>,
    pub wrap: bool,
}

impl Curve {
    pub fn eval(&self, t: f32) -> f32 {
        let len = self.points.len();
        if len == 0 {
            return 0.0;
        }
        if len == 1 {
            return self.points[0].1;
        }

        let x_min = self.points.first().unwrap().0;
        let x_max = self.points.last().unwrap().0;
        let t_normalized = t * (x_max - x_min) + x_min;

        if t_normalized <= x_min {
            if self.wrap {
                let (x0, y0) = self.points[len - 1];
                let (x1, y1) = self.points[0];
                return y0 + (t_normalized - x0) * (y1 - y0) / (x1 - x0);
            } else {
                return self.points[0].1;
            }
        }
        if t_normalized >= x_max {
            if self.wrap {
                let (x0, y0) = self.points[len - 1];
                let (x1, y1) = self.points[0];
                return y0 + (t_normalized - x0) * (y1 - y0) / (x1 - x0);
            } else {
                return self.points[len - 1].1;
            }
        }
        for i in 1..len {
            if t_normalized <= self.points[i].0 {
                let (x0, y0) = self.points[i - 1];
                let (x1, y1) = self.points[i];
                return y0 + (t_normalized - x0) * (y1 - y0) / (x1 - x0);
            }
        }

        0.0
    }
}

// impl Curve {
//     pub fn eval(&self, t: f32) -> f32 {
//         let len = self.points.len();
//         if len == 0 {
//             return 0.0;
//         }
//         if len == 1 {
//             return self.points[0].1;
//         }
//         if t <= self.points[0].0 {
//             if self.wrap {
//                 let (x0, y0) = self.points[len - 1];
//                 let (x1, y1) = self.points[0];
//                 return y0 + (t - x0) * (y1 - y0) / (x1 - x0);
//             } else {
//                 return self.points[0].1;
//             }
//         }
//         if t >= self.points[len - 1].0 {
//             if self.wrap {
//                 let (x0, y0) = self.points[len - 1];
//                 let (x1, y1) = self.points[0];
//                 return y0 + (t - x0) * (y1 - y0) / (x1 - x0);
//             } else {
//                 return self.points[len - 1].1;
//             }
//         }
//         for i in 1..len {
//             if t <= self.points[i].0 {
//                 let (x0, y0) = self.points[i - 1];
//                 let (x1, y1) = self.points[i];
//                 return y0 + (t - x0) * (y1 - y0) / (x1 - x0);
//             }
//         }
//         return 0.0;
//     }
// }

#[derive(Clone, Debug)]
pub struct ColorCurve {
    pub gradient: Vec<(Color, f32)>,
}

impl ColorCurve {
    pub fn new(gradient: Vec<(Color, f32)>) -> Self {
        Self { gradient }
    }

    pub fn eval(&self, t: f32) -> Color {
        if t <= 0.0 {
            return self.gradient[0].0;
        }
        if t >= 1.0 {
            return self.gradient.last().unwrap().0;
        }

        for i in 1..self.gradient.len() {
            let (prev_color, prev_pos) = self.gradient[i - 1];
            let (next_color, next_pos) = self.gradient[i];

            if t <= next_pos {
                let factor = (t - prev_pos) / (next_pos - prev_pos);
                return prev_color.lerp(next_color, factor);
            }
        }

        // This should never be reached, as t is already checked for >= 1.0
        return self.gradient.last().unwrap().0;
    }
}
