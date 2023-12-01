use comfy::*;
use comfy_core::image::GenericImageView;

simple_game!("EXR HDR Image", setup, update);

const TEST_IMAGES: &[&str] = &[
    "../Testing_Imagery/CC24_BT-709_uniform_1931.exr",
    "../Testing_Imagery/chinese_garden_2k.exr",
    "../Testing_Imagery/alexa_BT_709.exr",
    "../Testing_Imagery/blue_bar_709.exr",
];

fn setup(c: &mut EngineContext) {
    for path in TEST_IMAGES {
        // Test images from https://github.com/sobotka/Testing_Imagery
        let img =
            image::io::Reader::open(path).unwrap().decode().unwrap().flipv();

        let gc = &c.renderer.context;
        let label = *path;

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

    draw_sprite(
        texture_id(TEST_IMAGES[*CURRENT_IMAGE.lock()]),
        Vec2::ZERO,
        WHITE,
        1,
        viewport,
    );
}
