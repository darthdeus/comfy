use crate::*;
use fxhash::FxHashMap;

/// Experimental spatial hash.

#[derive(Copy, Clone, Debug)]
pub struct Intersection {
    pub point: Vec2,
    pub normal: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct AabbShape {
    pub min: Vec2,
    pub max: Vec2,
}

impl AabbShape {
    pub fn shape(center: Vec2, size: Vec2) -> Shape {
        let min = center - size / 2.0;
        let max = center + size / 2.0;
        Shape::Aabb(AabbShape { min, max })
    }

    pub fn intersects_circle(&self, circle: CircleShape) -> bool {
        let closest = self.min.max(self.max.min(circle.center));
        let distance = circle.center.distance(closest);
        distance <= circle.radius
    }

    pub fn intersects_aabb(&self, aabb: AabbShape) -> bool {
        self.min.x <= aabb.max.x &&
            self.max.x >= aabb.min.x &&
            self.min.y <= aabb.max.y &&
            self.max.y >= aabb.min.y
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn line_intersection(
        &self,
        start: Vec2,
        end: Vec2,
    ) -> Option<Intersection> {
        let dir = end - start;

        let mut tmin = (self.min.x - start.x) / dir.x;
        let mut tmax = (self.max.x - start.x) / dir.x;

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (self.min.y - start.y) / dir.y;
        let mut tymax = (self.max.y - start.y) / dir.y;

        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        tmin = tmin.max(tymin);
        tmax = tmax.min(tymax);

        if tmin > tmax {
            return None;
        }

        let t = if (0.0..=1.0).contains(&tmin) {
            tmin
        } else if (0.0..=1.0).contains(&tmax) {
            tmax
        } else {
            return None;
        };

        let intersection_point = start + dir * t;

        // Compute the normal
        let mut normal = Vec2::ZERO;

        // Determine which face was hit based on the intersection point
        let tolerance = 1e-5; // A small tolerance value to account for floating point errors

        if (intersection_point.x - self.min.x).abs() < tolerance {
            normal = Vec2::new(-1.0, 0.0);
        } else if (intersection_point.x - self.max.x).abs() < tolerance {
            normal = Vec2::new(1.0, 0.0);
        } else if (intersection_point.y - self.min.y).abs() < tolerance {
            normal = Vec2::new(0.0, -1.0);
        } else if (intersection_point.y - self.max.y).abs() < tolerance {
            normal = Vec2::new(0.0, 1.0);
        }

        Some(Intersection { point: intersection_point, normal })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CircleShape {
    pub center: Vec2,
    pub radius: f32,
}

impl CircleShape {
    pub fn bounding_rect(&self) -> AabbShape {
        let min = self.center - Vec2::splat(self.radius);
        let max = self.center + Vec2::splat(self.radius);
        AabbShape { min, max }
    }

    pub fn intersects_circle(&self, circle: CircleShape) -> bool {
        let distance = self.center.distance(circle.center);
        distance <= self.radius + circle.radius
    }

    pub fn intersects_aabb(&self, aabb: AabbShape) -> bool {
        aabb.intersects_circle(*self)
    }

    // pub fn intersects_line(&self, start: Vec2, end: Vec2) -> Option<Vec2> {
    //     let to_target = self.center - start;
    //
    //     let line_vec = end - start;
    //     let ray_len = line_vec.length();
    //     let ray_dir = line_vec.normalize();
    //
    //     let dot = to_target.dot(ray_dir);
    //
    //     if dot < 0.0 || dot > ray_len {
    //         return None;
    //     }
    //
    //     let closest_point = start + ray_dir * dot;
    //
    //     let dist_squared = (self.center - closest_point).length_squared();
    //
    //     if dist_squared > self.radius.powi(2) {
    //         return None;
    //     }
    //
    //     let t = (self.radius.powi(2) - dist_squared).sqrt();
    //
    //     let intersection1 = closest_point + ray_dir * (0.0 - t);
    //     let intersection2 = closest_point + ray_dir * t;
    //
    //     if (intersection1 - start).length() < (intersection2 - start).length() {
    //         Some(intersection1)
    //     } else {
    //         Some(intersection2)
    //     }
    // }

    pub fn intersects_line(
        &self,
        start: Vec2,
        end: Vec2,
    ) -> Option<Intersection> {
        let to_target = self.center - start;

        let line_vec = end - start;
        let ray_len = line_vec.length();
        let ray_dir = line_vec.normalize();

        let dot = to_target.dot(ray_dir);

        if dot < 0.0 || dot > ray_len {
            return None;
        }

        let closest_point = start + ray_dir * dot;

        let dist_squared = (self.center - closest_point).length_squared();

        if dist_squared > self.radius.powi(2) {
            return None;
        }

        let t = (self.radius.powi(2) - dist_squared).sqrt();

        let intersection1 = closest_point + ray_dir * (0.0 - t);
        let intersection2 = closest_point + ray_dir * t;

        let intersection_point = if (intersection1 - start).length() <
            (intersection2 - start).length()
        {
            intersection1
        } else {
            intersection2
        };

        // Calculate the normal at the intersection point
        let normal = (intersection_point - self.center).normalize();

        Some(Intersection { point: intersection_point, normal })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Circle(CircleShape),
    Aabb(AabbShape),
}

impl Shape {
    pub fn bounding_rect(&self) -> AabbShape {
        match self {
            Shape::Circle(circle) => circle.bounding_rect(),
            Shape::Aabb(aabb) => *aabb,
        }
    }

    pub fn intersects_shape(&self, shape: Shape) -> bool {
        match (*self, shape) {
            (Shape::Circle(circle1), Shape::Circle(circle2)) => {
                circle1.intersects_circle(circle2)
            }
            (Shape::Circle(circle), Shape::Aabb(aabb)) |
            (Shape::Aabb(aabb), Shape::Circle(circle)) => {
                circle.intersects_aabb(aabb)
            }
            (Shape::Aabb(aabb1), Shape::Aabb(aabb2)) => {
                aabb1.intersects_aabb(aabb2)
            }
        }
    }

    pub fn intersects_line(
        &self,
        start: Vec2,
        end: Vec2,
    ) -> Option<Intersection> {
        match self {
            Shape::Circle(circle) => circle.intersects_line(start, end),
            Shape::Aabb(aabb) => aabb.line_intersection(start, end),
        }
    }
}

#[derive(Clone, Copy)]
pub enum SpatialQuery {
    ShapeQuery(Shape),
}

#[derive(Clone, Copy)]
pub struct SpatialHashData {
    pub shape: Shape,
    pub userdata: UserData,
}

pub struct SpatialHash {
    pub grid_size: f32,
    pub inner: FxHashMap<(i32, i32), Vec<SpatialHashData>>,
}

impl SpatialHash {
    pub fn new() -> Self {
        const DEFAULT_GRID_SIZE: f32 = 100.0;
        Self { grid_size: DEFAULT_GRID_SIZE, inner: FxHashMap::default() }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn add_shape(&mut self, shape: Shape, data: UserData) {
        match shape {
            Shape::Circle(circle) => {
                self.add_shape(Shape::Aabb(circle.bounding_rect()), data);
            }
            Shape::Aabb(aabb) => {
                let min = aabb.min / self.grid_size;
                let max = aabb.max / self.grid_size;
                let min = min.floor();
                let max = max.ceil();

                for x in min.x as i32..max.x as i32 {
                    for y in min.y as i32..max.y as i32 {
                        let key = (x, y);
                        let entry = self.inner.entry(key).or_default();

                        entry.push(SpatialHashData { shape, userdata: data });
                    }
                }
            }
        }
    }

    pub fn query(
        &self,
        query: SpatialQuery,
    ) -> impl Iterator<Item = &UserData> {
        match query {
            SpatialQuery::ShapeQuery(shape) => {
                let bounding_rect = shape.bounding_rect();
                let min = bounding_rect.min / self.grid_size;
                let max = bounding_rect.max / self.grid_size;
                let min = min.floor();
                let max = max.ceil();
                (min.x as i32..max.x as i32)
                    .flat_map(move |x| {
                        (min.y as i32..max.y as i32).map(move |y| (x, y))
                    })
                    .flat_map(move |key| {
                        self.inner.get(&key).into_iter().flatten()
                    })
                    .filter(move |data| data.shape.intersects_shape(shape))
                    .map(|data| &data.userdata)
            }
        }
    }

    pub fn raycast(
        &self,
        start: Vec2,
        end: Vec2,
    ) -> Option<(Intersection, &UserData)> {
        let mut t = 0.0;
        // let dir = (end - start).normalize();
        let mut closest_intersection: Option<(Intersection, &UserData)> = None;

        while t <= 1.0 {
            let current_point = start + t * (end - start);
            let key = (
                (current_point.x / self.grid_size).floor() as i32,
                (current_point.y / self.grid_size).floor() as i32,
            );

            if let Some(cell) = self.inner.get(&key) {
                for spatial_data in cell {
                    if let Some(intersection) =
                        spatial_data.shape.intersects_line(start, end)
                    {
                        if closest_intersection.map_or(
                            true,
                            |(closest_point, _)| {
                                intersection.point.distance_squared(start) <
                                    closest_point
                                        .point
                                        .distance_squared(start)
                            },
                        ) {
                            closest_intersection =
                                Some((intersection, &spatial_data.userdata));
                        }
                    }
                }
            }

            t += self.grid_size / (end - start).length();
        }

        // draw_text(
        //     &format!("{:.1?} {:.1?} {:#.1?}", start, end, closest_intersection),
        //     start,
        //     WHITE,
        //     TextAlign::Center,
        // );

        closest_intersection
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct UserData {
    pub entity_type: u64,
    pub entity: Option<Entity>,
}

pub fn draw_spatial(spatial: &SpatialHash) {
    for (_, bucket) in spatial.inner.iter() {
        for item in bucket.iter() {
            match &item.shape {
                Shape::Circle(_) => todo!(),
                Shape::Aabb(aabb) => {
                    draw_rect_outline(
                        aabb.center(),
                        aabb.size(),
                        0.1,
                        RED,
                        499,
                    );
                }
            }
        }
    }
}
