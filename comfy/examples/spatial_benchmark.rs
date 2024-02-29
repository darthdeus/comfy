use comfy::*;
use comfy_core::spatial_hash::*;

fn main() {
    let mut spatial = SpatialHash::new();

    loop {
        let now = Instant::now();

        // insert 1000 entities
        for _ in 0..1000 {
            // random vec
            let vec = vec2(random() * 20.0, random() * 20.0);

            spatial.add_shape(
                Shape::Circle(CircleShape {
                    center: vec,
                    radius: random() * 2.0,
                }),
                UserData { entity_type: 0, entity: None },
            );
        }

        // total
        let mut total = 0;

        for cell in spatial.inner.iter() {
            total += cell.1.len();
        }

        let mut count = 0;

        // query 1000 times
        for _ in 0..1000 {
            let vec = vec2(random() * 20.0, random() * 20.0);

            let result = spatial.query(SpatialQuery::ShapeQuery(
                Shape::Circle(CircleShape { center: vec, radius: 2.0 }),
            ));

            count += result.count(); // assuming result is a Vec or similar collection
        }


        println!(
            "Count: {} ... {}us ... cells: {} ... average per cell: {}",
            count,
            now.elapsed().as_micros(),
            spatial.inner.len(),
            total / usize::max(spatial.inner.len(), 1)
        );

        spatial.clear();
    }
}
