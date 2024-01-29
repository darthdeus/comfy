use comfy_core::perf_counter;
use image::RgbaImage;

use crate::*;

#[derive(Copy, Clone, Debug)]
pub struct ScreenshotParams {
    pub record_screenshots: bool,
    /// When set to 1, a screenshot will be taken every frame.
    /// When set to a higher number, a screenshot will be taken every n frames.
    pub screenshot_interval_n: usize,
    pub history_length: usize,

    counter: usize,
}

impl Default for ScreenshotParams {
    fn default() -> Self {
        Self {
            record_screenshots: false,
            screenshot_interval_n: 1,
            history_length: 10,
            counter: 0,
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn record_screenshot_history(
    _screen: UVec2,
    _context: &GraphicsContext,
    _screenshot_buffer: &SizedBuffer,
    _output: &wgpu::SurfaceTexture,
    _params: &mut ScreenshotParams,
    _screenshot_history_buffer: &mut VecDeque<RgbaImage>,
) {
}

#[cfg(not(target_arch = "wasm32"))]
pub fn record_screenshot_history(
    screen: UVec2,
    context: &GraphicsContext,
    screenshot_buffer: &SizedBuffer,
    output: &wgpu::SurfaceTexture,
    params: &mut ScreenshotParams,
    screenshot_history_buffer: &mut VecDeque<RgbaImage>,
) {
    let start_time = Instant::now();

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

    let screenshot_image = pollster::block_on(async {
        let buffer_slice = screenshot_buffer.buffer.slice(..);

        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        context.device.poll(wgpu::Maintain::Wait);
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

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

        resized
    });

    if params.counter % params.screenshot_interval_n == 0 {
        if screenshot_history_buffer.len() == params.history_length {
            screenshot_history_buffer.pop_front();
        }

        screenshot_history_buffer.push_back(screenshot_image);
    }

    params.counter += 1;

    perf_counter(
        "screenshots in buffer",
        screenshot_history_buffer.len() as u64,
    );
    perf_counter("screenshot time", start_time.elapsed().as_micros() as u64);

    screenshot_buffer.buffer.unmap();
}

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
