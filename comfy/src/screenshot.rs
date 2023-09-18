use crate::*;

// fn save_gif(
//     path: &str,
//     frames: &mut Vec<Vec<u8>>,
//     speed: i32,
//     size: u16,
// ) -> Result<(), failure::Error> {
//     use gif::{Encoder, Frame, Repeat, SetParameter};
//
//     let mut image = std::fs::File::create(path)?;
//     let mut encoder = Encoder::new(&mut image, size, size, &[])?;
//     encoder.set(Repeat::Infinite)?;
//
//     for mut frame in frames {
//         encoder.write_frame(&Frame::from_rgba_speed(
//             size, size, &mut frame, speed,
//         ))?;
//     }
//
//     Ok(())
// }
