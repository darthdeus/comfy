use comfy::*;
use comfy_core::image::GenericImageView;

// This example requires the test images from https://github.com/sobotka/Testing_Imagery
// to be available in a directory next to the Comfy repo (as a sibling). We don't embed these in
// Comfy since the files are quite bit.
simple_game!("EXR HDR Image", setup, update);

const TEST_IMAGES: &[&str] = &[
    // Compare to
    // https://upload.wikimedia.org/wikipedia/commons/thumb/f/f1/TestScreen_square_more_colors.svg/1024px-TestScreen_square_more_colors.svg.png
    "assets/color-test-1024px.png",
    // All of these are from the Testing_Imagery repo linked at the top.
    "../Testing_Imagery/CC24_BT-709_uniform_1931.exr",
    "../Testing_Imagery/chinese_garden_2k.exr",
    "../Testing_Imagery/alexa_BT_709.exr",
    "../Testing_Imagery/blue_bar_709.exr",
];

fn setup(c: &mut EngineContext) {
    for path in TEST_IMAGES {
        let img =
            image::io::Reader::open(path).unwrap().decode().unwrap().flipv();

        let gc = &c.renderer.context;
        let label = *path;

        let (img, texture) = if path.ends_with(".exr") {
            let f16_data = img
                .to_rgba32f()
                .iter()
                .copied()
                .map(half::f16::from_f32)
                .collect_vec();

            let px_size = 4 * std::mem::size_of::<half::f16>();

            let texture = Texture::from_image_data_with_format(
                &gc.device,
                &gc.queue,
                bytemuck::cast_slice(&f16_data),
                Some(label),
                wgpu::AddressMode::ClampToEdge,
                wgpu::TextureFormat::Rgba16Float,
                img.dimensions(),
                px_size as u32,
            )
            .unwrap();

            (img, texture)
        } else {
            let data = img.to_rgba8();
            let px_size = 4 * std::mem::size_of::<u8>();

            let texture = Texture::from_image_data_with_format(
                &gc.device,
                &gc.queue,
                bytemuck::cast_slice(&data),
                Some(label),
                wgpu::AddressMode::ClampToEdge,
                wgpu::TextureFormat::Rgba8Unorm,
                img.dimensions(),
                px_size as u32,
            )
            .unwrap();

            (img, texture)
        };

        load_texture_with_image(
            &c.renderer.context,
            label,
            img,
            texture,
            &mut c.renderer.textures.lock(),
        );
    }
}

static CURRENT_IMAGE: Mutex<usize> = Mutex::new(0);

fn update(_c: &mut EngineContext) {
    if is_key_pressed(KeyCode::Space) {
        let mut config = game_config_mut();
        config.tonemapping_enabled = !config.tonemapping_enabled;
    }

    if is_key_pressed(KeyCode::W) {
        let mut current_image = CURRENT_IMAGE.lock();

        *current_image = if *current_image == 0 {
            TEST_IMAGES.len() - 1
        } else {
            *current_image - 1
        };
    }

    if is_key_pressed(KeyCode::S) {
        let mut current_image = CURRENT_IMAGE.lock();
        *current_image = (*current_image + 1) % TEST_IMAGES.len();
    }

    let viewport = main_camera().world_viewport();

    clear_background(Color::gray(0.1));

    let size = if *CURRENT_IMAGE.lock() == 0 {
        vec2(viewport.y, viewport.y)
    } else {
        viewport
    };

    draw_sprite(
        texture_id(TEST_IMAGES[*CURRENT_IMAGE.lock()]),
        Vec2::ZERO,
        WHITE,
        1,
        size,
    );
}
