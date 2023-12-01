use comfy::*;
use comfy_core::image::GenericImageView;

simple_game!("EXR HDR Image", setup, update);

fn setup(c: &mut EngineContext) {
    // Test images from https://github.com/sobotka/Testing_Imagery
    let img = image::io::Reader::open(
        // "../Testing_Imagery/CC24_BT-709_uniform_1931.exr",
        // "../Testing_Imagery/chinese_garden_2k.exr",
        "../Testing_Imagery/alexa_BT_709.exr",
        // "../Testing_Imagery/blue_bar_709.exr",
    )
    .unwrap()
    .decode()
    .unwrap()
    .flipv();

    let gc = &c.renderer.context;
    let label = "test-img";

    let f16_data =
        img.to_rgba32f().iter().copied().map(half::f16::from_f32).collect_vec();

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

fn update(_c: &mut EngineContext) {
    clear_background(Color::gray(0.1));

    let viewport = main_camera().world_viewport();

    draw_sprite(texture_id("test-img"), Vec2::ZERO, WHITE, 1, viewport);
}
