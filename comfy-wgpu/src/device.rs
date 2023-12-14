use crate::*;

pub async fn create_graphics_context(window: &Window) -> GraphicsContext {
    let size = window.inner_size();

    let backends =
        wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        // backends: wgpu::Backends::GL,
        backends,
        dx12_shader_compiler: Default::default(),
        // TODO: make validation configurable?
        flags: if cfg!(debug_assertions) {
            wgpu::InstanceFlags::debugging()
        } else {
            wgpu::InstanceFlags::VALIDATION
        },
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    let surface = unsafe {
        instance.create_surface(&window).expect("surface config must be valid")
    };

    trace!("Requesting adapter");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            // TODO: make this configurable
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("adapter config must be valid");

    info!("Using adapter: {:?}", adapter.get_info().name);

    trace!("Requesting device");

    #[cfg(not(target_arch = "wasm32"))]
    let limits = wgpu::Limits {
        max_texture_dimension_2d: 4096,
        ..wgpu::Limits::downlevel_defaults()
    };

    #[cfg(target_arch = "wasm32")]
    let limits = wgpu::Limits {
        max_texture_dimension_2d: 4096,
        ..wgpu::Limits::downlevel_webgl2_defaults()
    };

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits,
                label: None,
            },
            None,
        )
        .await
        .expect("failed to create wgpu adapter");

    #[cfg(fature = "ci-release")]
    device.on_uncaptured_error(Box::new(|err| {
        error!("WGPU ERROR: {:?}", err);
        panic!("Exiting due to wgpu error: {:?}", err);
    }));

    let caps = surface.get_capabilities(&adapter);
    let supported_formats = caps.formats;
    info!("Supported formats: {:?}", supported_formats);

    #[cfg(not(target_arch = "wasm32"))]
    let preferred_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    #[cfg(target_arch = "wasm32")]
    let preferred_format = wgpu::TextureFormat::Rgba8UnormSrgb;

    let monitor_surface_format =
        if supported_formats.contains(&preferred_format) {
            preferred_format
        } else {
            let fallback = supported_formats[0];

            error!(
                "Unsupported preferred surface format: {:?}. Using first \
                 supported format: {:?}",
                preferred_format, fallback
            );

            fallback
        };

    #[cfg(feature = "record-pngs")]
    let surface_usage =
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC;
    #[cfg(not(feature = "record-pngs"))]
    let surface_usage = wgpu::TextureUsages::RENDER_ATTACHMENT;


    let desired_present_mode =
        match std::env::var("COMFY_VSYNC_OVERRIDE").as_deref() {
            Ok("0") | Ok("f") | Ok("false") => {
                info!("VSYNC OVERRIDE via env var, set to VSYNC=off");
                wgpu::PresentMode::Immediate
            }
            Ok("1") | Ok("t") | Ok("true") => {
                info!("VSYNC OVERRIDE via env var, set to VSYNC=on");
                wgpu::PresentMode::AutoVsync
            }
            _ => {
                if game_config().vsync_enabled {
                    wgpu::PresentMode::AutoVsync
                } else {
                    wgpu::PresentMode::AutoNoVsync
                }
            }
        };

    let present_mode = if caps.present_modes.contains(&desired_present_mode) {
        desired_present_mode
    } else {
        caps.present_modes[0]
    };

    info!("ACTUAL PRESENT MODE: {:?}", present_mode);

    let config = wgpu::SurfaceConfiguration {
        usage: surface_usage,
        format: monitor_surface_format,
        width: size.width,
        height: size.height,
        present_mode,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
    };

    trace!("Configuring surface");

    surface.configure(&device, &config);

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true,
                        },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::Filtering,
                    ),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

    let textures = Arc::new(Mutex::new(HashMap::new()));

    let device = Arc::new(device);
    let queue = Arc::new(queue);
    let texture_layout = Arc::new(texture_bind_group_layout);

    let texture_creator = Arc::new(AtomicRefCell::new(WgpuTextureCreator {
        textures: textures.clone(),
        layout: texture_layout.clone(),
        queue: queue.clone(),
        device: device.clone(),
    }));

    GraphicsContext {
        surface: Arc::new(surface),
        instance: Arc::new(instance),
        adapter: Arc::new(adapter),
        device,
        queue,
        texture_layout,
        config: Arc::new(AtomicRefCell::new(config)),
        texture_creator,
        textures,
    }
}
