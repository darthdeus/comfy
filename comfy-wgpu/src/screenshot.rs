#[cfg(feature = "record-pngs")]
use crate::*;

#[cfg(feature = "record-pngs")]
pub fn record_pngs(
    screen: UVec2,
    context: &GraphicsContext,
    screenshot_buffer: &SizedBuffer,
    output: &wgpu::SurfaceTexture,
) {
    {
        let mut encoder = context.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Copy output texture Encoder"),
            },
        );

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &output.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &screenshot_buffer.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        std::mem::size_of::<u32>() as u32 * screen.x,
                    ),
                    rows_per_image: Some(screen.y),
                },
            },
            output.texture.size(),
        );

        context.queue.submit(std::iter::once(encoder.finish()));
    }

    pollster::block_on(async {
        let buffer_slice = screenshot_buffer.buffer.slice(..);

        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        context.device.poll(wgpu::Maintain::Wait);
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        let path = std::env::current_exe().unwrap();
        let example_name = path.file_stem().unwrap().to_string_lossy();

        let images_dir = format!("target/images/{}", example_name);

        let videos_dir = "target/videos";
        let screenshots_dir = "target/screenshots";

        std::fs::create_dir_all(&images_dir).unwrap();
        std::fs::create_dir_all(&videos_dir).unwrap();
        std::fs::create_dir_all(&screenshots_dir).unwrap();

        let name = format!("{}/image-{:03}.png", &images_dir, get_frame());

        {
            let mut rgba_data: Vec<u8> = Vec::with_capacity(data.len());

            for chunk in data.chunks_exact(4) {
                let b = chunk[0];
                let g = chunk[1];
                let r = chunk[2];
                let a = chunk[3];

                rgba_data.push(r);
                rgba_data.push(g);
                rgba_data.push(b);
                rgba_data.push(a);
            }


            let buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                screen.x, screen.y, rgba_data,
            )
            .unwrap();

            let resized = image::imageops::resize(
                &buffer,
                screen.x / 3,
                screen.y / 3,
                image::imageops::FilterType::Nearest,
            );

            resized.save(name).unwrap();

            if get_frame() > 60 {
                resized
                    .save(format!("{}/{}.png", screenshots_dir, example_name))
                    .unwrap();

                let ffmpeg_command = "ffmpeg";
                let framerate = "30";
                let input_pattern = "image-%03d.png";
                let output_file =
                    format!("{}/{}.webm", videos_dir, example_name);

                let output = std::process::Command::new(ffmpeg_command)
                    .arg("-framerate")
                    .arg(framerate)
                    .arg("-y")
                    .arg("-i")
                    .arg(format!("{}/{}", images_dir, input_pattern))
                    .arg(output_file)
                    .output()
                    .expect("Failed to execute ffmpeg command");

                if output.status.success() {
                    println!("Successfully generated the GIF.");
                } else {
                    println!("Error generating the GIF:");
                    println!(
                        "stdout: {}",
                        String::from_utf8_lossy(&output.stdout)
                    );
                    println!(
                        "stderr: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }

                std::process::exit(0);
            }
        }
    });

    screenshot_buffer.buffer.unmap();
}
