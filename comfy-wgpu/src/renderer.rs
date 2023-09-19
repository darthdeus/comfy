use std::sync::mpsc::{channel, Receiver, Sender};

use crate::*;

use image::Rgba;
use winit::{dpi::PhysicalSize, window::Window};

pub type PipelineMap = HashMap<String, wgpu::RenderPipeline>;
pub type TextureMap = HashMap<TextureHandle, (wgpu::BindGroup, Texture)>;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadUniform {
    pub clip_position: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Clone, Debug)]
pub struct Shader {
    pub name: String,
    pub source: String,
}

impl Shader {
    pub fn to_wgpu<'a>(&'a self) -> wgpu::ShaderModuleDescriptor<'a> {
        wgpu::ShaderModuleDescriptor {
            label: Some(&self.name),
            source: wgpu::ShaderSource::Wgsl(self.source.as_str().into()),
        }
    }
}

pub type ShaderMap = HashMap<Cow<'static, str>, Shader>;

#[derive(Clone)]
pub struct GraphicsContext {
    pub instance: Arc<wgpu::Instance>,
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    // Shared for all regular textures/sprites
    pub texture_layout: Arc<wgpu::BindGroupLayout>,
}

pub struct WgpuRenderer {
    pub context: GraphicsContext,

    pub surface: wgpu::Surface,
    #[cfg(not(any(feature = "ci-release", target_arch = "wasm32")))]
    pub hot_reload: HotReload,

    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    pub pipelines: PipelineMap,
    pub shaders: RefCell<ShaderMap>,

    pub egui_winit: egui_winit::State,
    pub egui_render_routine: RefCell<EguiRenderRoutine>,
    pub egui_ctx: egui::Context,

    pub screenshot_buffer: SizedBuffer,

    pub vertex_buffer: SizedBuffer,
    pub index_buffer: SizedBuffer,

    pub quad_ubg: UniformBindGroup,

    pub texture_layout: Arc<wgpu::BindGroupLayout>,

    pub window: Window,

    pub depth_texture: Arc<Texture>,

    pub first_pass_texture: Texture,
    pub first_pass_bind_group: wgpu::BindGroup,

    pub lights_buffer: wgpu::Buffer,
    pub lights_bind_group: wgpu::BindGroup,
    pub lights_bind_group_layout: wgpu::BindGroupLayout,

    pub global_lighting_params_buffer: wgpu::Buffer,
    pub global_lighting_params_bind_group: Arc<wgpu::BindGroup>,
    pub global_lighting_params_bind_group_layout: wgpu::BindGroupLayout,

    pub bloom: Bloom,
    pub post_processing_effects: RefCell<Vec<PostProcessingEffect>>,

    pub render_texture_format: wgpu::TextureFormat,

    pub tonemapping_texture: Texture,
    pub tonemapping_bind_group: wgpu::BindGroup,

    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,

    pub color: Color,

    pub enable_z_buffer: bool,

    pub texture_creator: Arc<AtomicRefCell<WgpuTextureCreator>>,

    #[cfg(not(target_arch = "wasm32"))]
    pub thread_pool: rayon::ThreadPool,
    pub rx_texture: Receiver<LoadedImage>,
    pub tx_texture: Sender<LoadedImage>,

    pub textures: Arc<Mutex<TextureMap>>,
}

impl WgpuRenderer {
    pub async fn new(window: Window, egui_winit: egui_winit::State) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(&window).unwrap() };

        trace!("Requesting adapter");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

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

        #[cfg(feature = "ci-release")]
        device.on_uncaptured_error(Box::new(|err| {
            error!("WGPU ERROR: {:?}", err);
            panic!("Exiting due to wgpu error: {:?}", err);
        }));

        let caps = surface.get_capabilities(&adapter);
        let supported_formats = caps.formats;
        info!("Supported formats: {:?}", supported_formats);

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

        let render_texture_format = wgpu::TextureFormat::Rgba16Float;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT |
                wgpu::TextureUsages::COPY_SRC,
            format: monitor_surface_format,
            width: size.width,
            height: size.height,
            #[cfg(not(target_arch = "wasm32"))]
            present_mode: wgpu::PresentMode::Immediate,
            #[cfg(target_arch = "wasm32")]
            present_mode: wgpu::PresentMode::Fifo,
            // TODO: not in build?
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        trace!("Configuring surface");

        surface.configure(&device, &config);

        trace!("Loading builtin engine textures");

        let mut textures = HashMap::default();

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

        let context = GraphicsContext {
            instance: Arc::new(instance),
            adapter: Arc::new(adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
            texture_layout: Arc::new(texture_bind_group_layout),
        };

        macro_rules! load_engine_tex {
            ($name: literal) => {{
                load_texture_from_engine_bytes(
                    &context,
                    $name,
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../assets/",
                        $name,
                        ".png"
                    )),
                    &mut textures,
                    wgpu::AddressMode::Repeat,
                );
            }};
        }

        load_engine_tex!("error");
        load_engine_tex!("1px");
        load_engine_tex!("test-grid");

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&main_camera());

        let camera_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM |
                    wgpu::BufferUsages::COPY_DST,
            },
        );

        let camera_bind_group_layout = context.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX |
                        wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            },
        );

        let camera_bind_group =
            context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

        let lights_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Lights Buffer"),
                contents: bytemuck::cast_slice(&[LightUniform::default()]),
                usage: wgpu::BufferUsages::UNIFORM |
                    wgpu::BufferUsages::COPY_DST,
            },
        );

        let lights_bind_group_layout: wgpu::BindGroupLayout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX |
                        wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Lights Bind Group Layout"),
            });

        let lights_bind_group =
            context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &lights_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: lights_buffer.as_entire_binding(),
                }],
                label: Some("Lights Bind Group"),
            });

        let global_lighting_params_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Global Lighting Params Buffer"),
                contents: bytemuck::cast_slice(&[
                    GlobalLightingParams::default(),
                ]),
                usage: wgpu::BufferUsages::UNIFORM |
                    wgpu::BufferUsages::COPY_DST,
            },
        );

        let lut_dim = 2;

        let color_lut_texture = Texture::create_uninit(
            &context.device,
            lut_dim,
            lut_dim,
            Some("LUT"),
        )
        .unwrap();

        let lut_image =
            image::ImageBuffer::from_fn(lut_dim, lut_dim, |x, y| {
                if x == 0 && y == 0 {
                    Rgba([255, 0, 0, 255]) // Red
                } else if x == 1 && y == 0 {
                    Rgba([0, 255, 0, 255]) // Green
                } else if x == 0 && y == 1 {
                    Rgba([0, 0, 255, 255]) // Blue
                } else {
                    Rgba([255, 255, 255, 255]) // White
                }
            });

        let lut_width = lut_image.width();
        let lut_height = lut_image.height();

        let color_lut_data: Vec<f32> = lut_image
            .pixels()
            .flat_map(|rgba| {
                let r = rgba[0] as f32 / 255.0;
                let g = rgba[1] as f32 / 255.0;
                let b = rgba[2] as f32 / 255.0;
                let a = rgba[3] as f32 / 255.0;
                vec![r, g, b, a]
            })
            .collect();

        context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &color_lut_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(color_lut_data.as_slice()),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    lut_width * 4 * std::mem::size_of::<f32>() as u32,
                ),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: lut_width,
                height: lut_height,
                depth_or_array_layers: 1,
            },
        );

        let global_lighting_params_bind_group_layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Global Lighting Params Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });

        let global_lighting_params_bind_group =
            context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Global Lighting Params Bind Group"),
                layout: &global_lighting_params_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: global_lighting_params_buffer
                            .as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &color_lut_texture.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(
                            &color_lut_texture.sampler,
                        ),
                    },
                ],
            });

        let global_lighting_params_bind_group =
            Arc::new(global_lighting_params_bind_group);

        trace!("Initializing egui");

        let egui_render_routine = EguiRenderRoutine::new(
            &context.device,
            config.format,
            1,
            config.width,
            config.height,
            window.scale_factor() as f32,
        );

        let screenshot_buffer = SizedBuffer::new(
            "screenshot_buffer",
            &context.device,
            (size.width * size.height) as usize *
                std::mem::size_of::<u32>() as usize,
            BufferType::Read,
        );

        info!("Initializing with scale factor: {}", window.scale_factor());

        let egui_ctx = egui::Context::default();

        let fonts = egui::FontDefinitions::default();

        // TODO: expose this for easier user defined fonts
        #[allow(unused_macros)]
        macro_rules! load_font {
            ($family_name:literal, $file_name:literal) => {
                let family_name = $family_name.to_string();

                fonts.font_data.insert(
                    family_name.clone(),
                    egui::FontData::from_static(include_bytes!(concat!(
                        "../../assets/",
                        $file_name,
                        ".ttf"
                    ))),
                );

                fonts
                    .families
                    .insert(egui::FontFamily::Name($family_name.into()), vec![
                        family_name,
                    ]);
            };
        }

        egui_ctx.set_fonts(fonts);

        let textures = Arc::new(Mutex::new(textures));

        let texture_creator =
            Arc::new(AtomicRefCell::new(WgpuTextureCreator {
                textures: textures.clone(),
                layout: context.texture_layout.clone(),
                queue: context.queue.clone(),
                device: context.device.clone(),
            }));

        BLOOD_CANVAS
            .set(AtomicRefCell::new(BloodCanvas::new(texture_creator.clone())))
            .expect("failed to create glow blood canvas");


        let mut post_processing_effects = Vec::new();

        macro_rules! make_effect {
            ($name:literal) => {{
                let effect = PostProcessingEffect::new(
                    $name.to_string(),
                    &context.device,
                    &[
                        &context.texture_layout,
                        &global_lighting_params_bind_group_layout,
                    ],
                    &config,
                    render_texture_format,
                    reloadable_wgsl_fragment_shader!($name),
                );

                post_processing_effects.push(effect);
            }};
        }

        // simple_fragment_shader(
        //     concat!($name, " Post Processing Shader"),
        //     include_str!(
        //     concat!("../../assets/shaders/", $name, ".wgsl")
        //     // "../../assets/shaders/invert.wgsl"
        // )),

        // make_effect!("invert");
        // make_effect!("invert");
        // make_effect!("invert");

        // make_effect!("invert");
        // make_effect!("red");
        // make_effect!("blue");

        // make_effect!("darken");
        // make_effect!("red");
        // make_effect!("darken");
        // make_effect!("invert");
        // make_effect!("darken");
        // make_effect!("invert");
        // make_effect!("darken");

        // make_effect!("screen-shake");
        make_effect!("chromatic-aberration");
        // make_effect!("film-grain");

        // make_effect!("copy");
        // make_effect!("dither");
        make_effect!("copy");

        // make_effect!("palette");
        // make_effect!("invert");

        let first_pass_texture = Texture::create_scaled_surface_texture(
            &context.device,
            &config,
            1.0,
            "First Pass Texture",
        );

        let first_pass_bind_group = context.device.simple_bind_group(
            "First Pass Bind Group",
            &first_pass_texture,
            &context.texture_layout,
        );

        let tonemapping_texture = Texture::create_scaled_mip_surface_texture(
            &context.device,
            &config,
            wgpu::TextureFormat::Rgba16Float,
            1.0,
            1,
            "Tonemapping",
        );

        let tonemapping_bind_group = context.device.simple_bind_group(
            "Tonemapping Bind Group",
            &tonemapping_texture,
            &context.texture_layout,
        );

        let quad = QuadUniform { clip_position: [0.0, 0.0], size: [0.2, 0.2] };

        let quad_ubg = UniformBindGroup::simple(
            "Debug Quad",
            &context.device,
            bytemuck::cast_slice(&[quad]),
        );

        let bloom = Bloom::new(
            &context,
            &config,
            render_texture_format,
            global_lighting_params_bind_group.clone(),
            &global_lighting_params_bind_group_layout,
        );

        let vertex_buffer = SizedBuffer::new(
            "Mesh Vertex Buffer",
            &context.device,
            1024 * 1024,
            BufferType::Vertex,
        );

        let index_buffer = SizedBuffer::new(
            "Mesh Index Buffer",
            &context.device,
            1024 * 1024,
            BufferType::Index,
        );


        let (tx_texture, rx_texture) = channel::<LoadedImage>();

        // TODO: resize

        let depth_texture = Texture::create_depth_texture(
            &context.device,
            &config,
            "Depth Texture",
        );

        Self {
            surface,

            #[cfg(not(target_arch = "wasm32"))]
            thread_pool: rayon::ThreadPoolBuilder::new().build().unwrap(),

            rx_texture,
            tx_texture,

            depth_texture: Arc::new(depth_texture),

            pipelines: HashMap::new(),
            shaders: RefCell::new(load_shaders()),
            #[cfg(not(any(feature = "ci-release", target_arch = "wasm32")))]
            hot_reload: HotReload::new(),

            screenshot_buffer,

            vertex_buffer,
            index_buffer,

            config,
            size,

            post_processing_effects: RefCell::new(post_processing_effects),
            bloom,

            egui_winit,
            egui_render_routine: RefCell::new(egui_render_routine),
            egui_ctx,

            first_pass_texture,
            first_pass_bind_group,

            lights_buffer,
            lights_bind_group,
            lights_bind_group_layout,

            quad_ubg,

            global_lighting_params_buffer,
            global_lighting_params_bind_group,
            global_lighting_params_bind_group_layout,

            texture_layout: context.texture_layout.clone(),

            tonemapping_texture,
            tonemapping_bind_group,

            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,

            textures,
            render_texture_format,

            color: Color::new(0.1, 0.2, 0.3, 1.0),

            enable_z_buffer: false,

            texture_creator,

            window,

            context,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn render_post_processing(
        &mut self,
        screen_view: &wgpu::TextureView,
        params: &GlobalLightingParams,
    ) {
        let _span = span!("render_post_processing");
        let mut encoder =
            self.context.device.simple_encoder("Post Processing Encoder");

        let mut input_bind_group = &self.first_pass_bind_group;

        self.bloom.draw(
            &self.context.device,
            &self.texture_layout,
            &self.first_pass_bind_group,
            &mut encoder,
        );

        let post_processing_effects = self.post_processing_effects.borrow();

        let enabled_effects =
            post_processing_effects.iter().filter(|x| x.enabled).collect_vec();

        for (i, effect) in enabled_effects.iter().enumerate() {
            let output_texture_view = if i == enabled_effects.len() - 1 {
                &self.tonemapping_texture.view
            } else {
                &effect.render_texture.view
            };

            let maybe_pipeline = if self.pipelines.contains_key(&effect.name) {
                Some(self.pipelines.get(&effect.name).unwrap())
            } else {
                info!("Loading EFFECT: {}", effect.name);

                if let Some(shader) = self
                    .shaders
                    .borrow()
                    .get(&Into::<Cow<_>>::into(effect.name.clone()))
                {
                    let pipeline = create_post_processing_pipeline(
                        &effect.name,
                        &self.context.device,
                        self.render_texture_format,
                        &[
                            &self.texture_layout,
                            &self.global_lighting_params_bind_group_layout,
                        ],
                        shader.clone(),
                        // &effect.shader,
                        wgpu::BlendState::REPLACE,
                    );

                    self.pipelines.insert(effect.name.clone(), pipeline);
                    self.pipelines.get(&effect.name)
                } else {
                    None
                }
            };

            if let Some(pipeline) = maybe_pipeline {
                draw_post_processing_output(
                    &effect.name,
                    &mut encoder,
                    pipeline,
                    input_bind_group,
                    &self.global_lighting_params_bind_group,
                    // &effect.bind_group,
                    output_texture_view,
                    true,
                    None,
                );

                input_bind_group = &effect.bind_group;
            } else {
                error!("Missing shader for {}", effect.name);
            }
        }

        self.bloom.blit_final(
            &mut encoder,
            &self.tonemapping_texture.view,
            params,
        );

        let tonemapping_pipeline =
            self.pipelines.entry("tonemapping".into()).or_insert_with(|| {
                create_post_processing_pipeline(
                    "Tonemapping",
                    &self.context.device,
                    self.config.format,
                    &[
                        &self.texture_layout,
                        &self.global_lighting_params_bind_group_layout,
                    ],
                    reloadable_wgsl_fragment_shader!("tonemapping"),
                    wgpu::BlendState::REPLACE,
                )
            });


        {
            let should_clear = false;

            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Tonemapping"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: screen_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: if should_clear {
                                    wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 1.0,
                                    })
                                } else {
                                    wgpu::LoadOp::Load
                                },
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                });

            render_pass.set_pipeline(tonemapping_pipeline);
            render_pass.set_bind_group(0, &self.tonemapping_bind_group, &[]);
            render_pass.set_bind_group(
                1,
                &self.global_lighting_params_bind_group,
                &[],
            );

            render_pass.draw(0..3, 0..1);
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render_debug(&mut self, surface_view: &wgpu::TextureView) {
        let _span = span!("render_debug");

        let mut bind_groups = vec![&self.first_pass_bind_group];
        bind_groups.push(&self.bloom.threshold.bind_group);
        bind_groups.push(&self.bloom.blur_bind_group);

        let size = 0.3;
        let post_processing_effects = self.post_processing_effects.borrow();

        for effect in post_processing_effects.iter() {
            if effect.enabled {
                bind_groups.push(&effect.bind_group);
            }
        }

        let quads: Vec<_> = bind_groups
            .iter()
            .enumerate()
            .map(|(i, _)| {
                QuadUniform {
                    clip_position: [
                        1.0 - size / 2.0, // - size / 3.0 * i as f32,
                        0.8 - size / 2.0 - 2.0 * size * i as f32,
                    ],
                    size: [size, size],
                }
            })
            .collect();

        let debug_render_pipeline = self
            .pipelines
            .entry(
                if self.enable_z_buffer { "debug-z" } else { "debug" }.into(),
            )
            .or_insert_with(|| {
                create_render_pipeline_with_layout(
                    "Debug",
                    &self.context.device,
                    self.config.format,
                    &[&self.texture_layout, &self.quad_ubg.layout],
                    &[],
                    reloadable_wgsl_shader!("debug"),
                    BlendMode::Alpha,
                    self.enable_z_buffer,
                )
            });


        for (i, bind_group) in bind_groups.iter().enumerate() {
            self.context.queue.write_buffer(
                &self.quad_ubg.buffer,
                0,
                bytemuck::cast_slice(&[quads[i]]),
            );

            let mut encoder =
                self.context.device.simple_encoder("Debug Render Encoder");
            {
                // let mut render_pass = encoder.simple_render_pass(
                //     "Debug Render Pass",
                //     None,
                //     surface_view,
                // );

                let mut render_pass =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Debug Render Pass"),
                        color_attachments: &[Some(
                            wgpu::RenderPassColorAttachment {
                                view: surface_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: color_to_clear_op(None),
                                    store: true,
                                },
                            },
                        )],
                        depth_stencil_attachment: depth_stencil_attachment(
                            self.enable_z_buffer,
                            &self.depth_texture.view,
                            false,
                        ),
                    });


                render_pass.set_pipeline(debug_render_pipeline);
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_bind_group(1, &self.quad_ubg.bind_group, &[]);
                render_pass.draw(0..6, 0..1);
            }

            self.context.queue.submit(std::iter::once(encoder.finish()));
        }
    }

    pub fn render_egui(&self, view: &wgpu::TextureView) {
        let _span = span!("render_egui");

        let mut encoder =
            self.context.device.simple_encoder("egui Render Encoder");

        let paint_jobs =
            self.egui_render_routine.borrow_mut().end_frame_and_render(
                &self.egui_ctx,
                &self.context.device,
                &self.context.queue,
                &mut encoder,
            );

        let egui_render = self.egui_render_routine.borrow();
        let egui_render = &egui_render;

        {
            let mut render_pass =
                encoder.simple_render_pass("egui Render Pass", None, view);

            egui_render.render_pass.render(
                &mut render_pass,
                &paint_jobs,
                &egui_render.screen_descriptor,
            );
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn on_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.egui_winit.on_event(&self.egui_ctx, event).consumed
    }

    pub fn render_meshes(
        &mut self,
        is_first: bool,
        clear_color: Color,
        pass_data: MeshDrawData,
        surface_view: &wgpu::TextureView,
    ) {
        let _span = span!("render_meshes");

        let target_view = if self
            .post_processing_effects
            .borrow()
            .iter()
            .any(|x| x.enabled)
        {
            &self.first_pass_texture.view
        } else {
            surface_view
        };

        let textures = self.textures.lock();

        let _span = span!("blend_mode");

        let mesh_pipeline = {
            let name = format!(
                "Mesh {:?} {:?}",
                pass_data.blend_mode, self.enable_z_buffer
            );

            self.pipelines.entry(name.clone().into()).or_insert_with(|| {
                create_render_pipeline_with_layout(
                    &name,
                    &self.context.device,
                    wgpu::TextureFormat::Rgba16Float,
                    &[
                        &self.texture_layout,
                        &self.camera_bind_group_layout,
                        &self.lights_bind_group_layout,
                        &self.global_lighting_params_bind_group_layout,
                    ],
                    &[SpriteVertex::desc()],
                    reloadable_wgsl_shader!("sprite"),
                    pass_data.blend_mode,
                    self.enable_z_buffer,
                )
            })
        };

        perf_counter_inc("batch-count", 1);

        let tex_handle = pass_data.texture;
        let _span = span!("texture");
        // let tex_handle = TextureHandle::from_path("1px");

        let mut all_vertices: Vec<SpriteVertex> = vec![];
        let mut all_indices = vec![];

        for draw in pass_data.data.into_iter() {
            // for draw in group {
            // let mut mesh = draw.mesh.clone();

            // all_indices.extend(
            //     mesh.indices
            //         .iter()
            //         .cloned()
            //         .map(|x| x as u32 + all_vertices.len() as u32),
            // );
            //
            // all_vertices.extend(mesh.vertices.drain(..));

            let offset = all_vertices.len() as u32;
            all_vertices.extend(&draw.mesh.vertices);
            all_indices.extend(draw.mesh.indices.iter().map(|x| *x + offset));
        }

        // let all_vertices = mesh_draw[0].mesh.vertices.clone();
        // let all_indices = mesh_draw[0].mesh.indices.clone();

        let mut encoder =
            self.context.device.simple_encoder("Mesh Render Encoder");

        self.vertex_buffer.ensure_size_and_copy(
            &self.context.device,
            &self.context.queue,
            bytemuck::cast_slice(all_vertices.as_slice()),
        );

        self.index_buffer.ensure_size_and_copy(
            &self.context.device,
            &self.context.queue,
            bytemuck::cast_slice(all_indices.as_slice()),
        );

        {
            let clear_color = if is_first { Some(clear_color) } else { None };

            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Mesh Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: target_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: color_to_clear_op(clear_color),
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: depth_stencil_attachment(
                        self.enable_z_buffer,
                        &self.depth_texture.view,
                        is_first,
                    ),
                });

            render_pass.set_pipeline(mesh_pipeline);
            render_pass
                .set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));

            if !all_indices.is_empty() {
                render_pass.set_index_buffer(
                    self.index_buffer.buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
            }

            let tex_bind_group = &textures
                .get(&tex_handle)
                .unwrap_or_else(|| textures.get(&texture_id("error")).unwrap())
                .0;

            render_pass.set_bind_group(0, tex_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.lights_bind_group, &[]);
            render_pass.set_bind_group(
                3,
                &self.global_lighting_params_bind_group,
                &[],
            );

            if all_indices.is_empty() {
                render_pass.draw(0..all_vertices.len() as u32, 0..1);
            } else {
                render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);
            }
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render_particles(
        &mut self,
        is_first: bool,
        pass_data: ParticleDrawData,
        clear_color: Color,
        surface_view: &wgpu::TextureView,
    ) {
        let _span = span!("render_particles");

        let target_view = if self
            .post_processing_effects
            .borrow()
            .iter()
            .any(|x| x.enabled)
        {
            &self.first_pass_texture.view
        } else {
            surface_view
        };

        let textures = self.textures.lock();

        let particle_pipeline = {
            let name = format!(
                "Particle {:?} {:?}",
                pass_data.blend_mode, self.enable_z_buffer
            );

            self.pipelines.entry(name.clone().into()).or_insert_with(|| {
                create_render_pipeline_with_layout(
                    &name,
                    &self.context.device,
                    // self.config.format,
                    wgpu::TextureFormat::Rgba16Float,
                    &[
                        &self.texture_layout,
                        &self.camera_bind_group_layout,
                        &self.lights_bind_group_layout,
                        &self.global_lighting_params_bind_group_layout,
                    ],
                    &[SpriteVertex::desc()],
                    reloadable_wgsl_shader!("sprite"),
                    pass_data.blend_mode,
                    self.enable_z_buffer,
                )
            })
        };

        let mut all_vertices: Vec<SpriteVertex> = vec![];
        let mut all_indices: Vec<u32> = vec![];

        for draw in pass_data.data {
            let size = draw.size;

            let tex_size = ASSETS
                .borrow()
                .texture_image_map
                .lock()
                .get(&pass_data.texture)
                .map(|image| vec2(image.width() as f32, image.height() as f32))
                .unwrap_or(Vec2::ONE);

            let tex_width = tex_size.x;
            let tex_height = tex_size.y;

            let vertices = rotated_rectangle(
                // TODO: fix particle Z
                draw.position,
                RawDrawParams {
                    dest_size: Some(size),
                    rotation: draw.rotation,
                    source_rect: draw.source_rect,
                    ..Default::default()
                },
                tex_width,
                tex_height,
                draw.color,
                // TODO: scrolling particle offset?
                Vec2::ZERO,
            );

            let len = all_vertices.len() as u32;
            all_indices.extend_from_slice(&[
                len,
                2 + len,
                1 + len,
                len,
                3 + len,
                2 + len,
            ]);

            all_vertices.extend(vertices);
        }

        let mut encoder =
            self.context.device.simple_encoder("Particle Render Encoder");

        self.vertex_buffer.ensure_size_and_copy(
            &self.context.device,
            &self.context.queue,
            bytemuck::cast_slice(all_vertices.as_slice()),
        );

        self.index_buffer.ensure_size_and_copy(
            &self.context.device,
            &self.context.queue,
            bytemuck::cast_slice(all_indices.as_slice()),
        );

        {
            let clear_color = if is_first { Some(clear_color) } else { None };

            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Particle Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: target_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: color_to_clear_op(clear_color),
                                store: true,
                            },
                        },
                    )],
                    // depth_stencil_attachment: Some(
                    //     wgpu::RenderPassDepthStencilAttachment {
                    //         view: &self.depth_texture.view,
                    //         depth_ops: Some(wgpu::Operations {
                    //             load: clear_depth,
                    //             store: true,
                    //         }),
                    //         stencil_ops: None,
                    //     },
                    // ),
                    depth_stencil_attachment: depth_stencil_attachment(
                        self.enable_z_buffer,
                        &self.depth_texture.view,
                        is_first,
                    ),
                });

            render_pass.set_pipeline(particle_pipeline);
            render_pass
                .set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));

            if !all_indices.is_empty() {
                render_pass.set_index_buffer(
                    self.index_buffer.buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
            }

            let tex_bind_group = &textures
                .get(&pass_data.texture)
                .unwrap_or_else(|| textures.get(&texture_id("error")).unwrap())
                .0;

            render_pass.set_bind_group(0, tex_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.lights_bind_group, &[]);
            render_pass.set_bind_group(
                3,
                &self.global_lighting_params_bind_group,
                &[],
            );

            if all_indices.is_empty() {
                render_pass.draw(0..all_vertices.len() as u32, 0..1);
            } else {
                render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);
            }
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    pub fn begin_frame(&mut self) {
        self.egui_ctx
            .begin_frame(self.egui_winit.take_egui_input(&self.window));
    }

    pub fn update(&mut self, params: &mut DrawParams) {
        let _span = span!("renderer update");

        let mut changed_recording_mode = false;

        if is_key_pressed(KeyCode::F3) {
            params.config.dev.recording_mode =
                match params.config.dev.recording_mode {
                    RecordingMode::None => RecordingMode::Landscape,
                    RecordingMode::Tiktok => RecordingMode::Landscape,
                    RecordingMode::Landscape => RecordingMode::None,
                };

            changed_recording_mode = true;
        }

        if is_key_pressed(KeyCode::F9) {
            self.enable_z_buffer = !self.enable_z_buffer;
            info!("Z Buffer: {}", self.enable_z_buffer);
        }

        if is_key_pressed(KeyCode::F4) {
            params.config.dev.recording_mode =
                match params.config.dev.recording_mode {
                    RecordingMode::None => RecordingMode::Tiktok,
                    RecordingMode::Tiktok => RecordingMode::None,
                    RecordingMode::Landscape => RecordingMode::Tiktok,
                };

            changed_recording_mode = true;
        }


        // Load textures
        {
            while let Ok(loaded_image) = self.rx_texture.try_recv() {
                let context = self.context.clone();
                let textures = self.textures.clone();
                let tbgl = self.texture_layout.clone();

                // let context_inner = context.clone();
                // let tbgl = texture_bind_group_layout.clone();
                // let textures_inner = textures.clone();

                let texture_loop = || {
                    let texture = Texture::from_image(
                        &context.device,
                        &context.queue,
                        &loaded_image.image,
                        Some(&loaded_image.path),
                        false,
                    )
                    .unwrap();

                    let bind_group = context.device.simple_bind_group(
                        &format!("{}_bind_group", loaded_image.path),
                        &texture,
                        &tbgl,
                    );

                    textures
                        .lock()
                        .insert(loaded_image.handle, (bind_group, texture));
                };

                #[cfg(target_arch = "wasm32")]
                texture_loop();

                #[cfg(not(target_arch = "wasm32"))]
                self.thread_pool.install(texture_loop);
            }
        }

        if changed_recording_mode {
            info!("Recording Mode: {:?}", params.config.dev.recording_mode);

            self.window.set_title(&format!(
                "NANOVOID {} (COMFY ENGINE)",
                if params.config.dev.recording_mode == RecordingMode::Tiktok {
                    "Portrait"
                } else {
                    ""
                },
            ));

            let ratio = 1920.0 / 1080.0;

            let size = 600.0;
            let portrait_res = winit::dpi::PhysicalSize::new(
                size as i32,
                (size * ratio) as i32,
            );

            let landscape_res = winit::dpi::PhysicalSize::new(1920, 1080);

            let (resolution, _ui_toggle) =
                match params.config.dev.recording_mode {
                    RecordingMode::None => (landscape_res, true),
                    RecordingMode::Tiktok => (portrait_res, false),
                    RecordingMode::Landscape => (landscape_res, false),
                };

            self.window.set_inner_size(resolution);
            // self.window.center();
        }

        #[cfg(not(any(feature = "ci-release", target_arch = "wasm32")))]
        if self.hot_reload.maybe_reload_shaders() {
            // TODO: this breaks previously loaded user shaders
            error!("TODO: reload breaks previously loaded user shaders");
            self.shaders = RefCell::new(load_shaders());
            self.pipelines.clear();
        }

        self.camera_uniform.update_view_proj(&main_camera());

        params.lighting.time = get_time() as f32;
        {
            let camera = main_camera();
            params.lighting.chromatic_aberration =
                (camera.shake_amount * 200.0 * camera.shake_timer)
                    .clamp(0.0, 200.0);
        }

        self.context.queue.write_buffer(
            &self.global_lighting_params_buffer,
            0,
            bytemuck::cast_slice(&[*params.lighting]),
        );

        self.context.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let mut light_uniform = LightUniform::default();

        for (i, light) in params.lights.iter().enumerate() {
            if i >= 128 {
                break;
            }

            light_uniform.lights[i] = *light;
            light_uniform.num_lights += 1;
        }

        self.context.queue.write_buffer(
            &self.lights_buffer,
            0,
            bytemuck::cast_slice(&[light_uniform]),
        );
    }

    pub fn draw(&mut self, params: DrawParams) {
        span_with_timing!("render");

        let output = {
            let _span = span!("get current surface");
            self.surface.get_current_texture().unwrap()
        };

        let surface_view = {
            let _span = span!("create surface view");
            output.texture.create_view(&wgpu::TextureViewDescriptor::default())
        };

        let render_passes = collect_render_passes(&params);

        perf_counter("render pass blocks", render_passes.len() as u64);
        perf_counter(
            "mesh draws",
            render_passes
                .iter()
                .filter(|x| matches!(x.data, DrawData::Meshes(_)))
                .count() as u64,
        );

        perf_counter(
            "particle draws",
            render_passes
                .iter()
                .filter(|x| matches!(x.data, DrawData::Particles(_)))
                .count() as u64,
        );

        let mut is_first = true;

        let grouped_render_passes = render_passes
            .into_iter()
            .sorted_by_key(|p| p.z_index)
            .group_by(|p| p.z_index);

        for (_, z_index_group) in &grouped_render_passes {
            for (blend_mode, blend_group) in &z_index_group
                .sorted_by_key(|x| x.blend_mode)
                .group_by(|x| x.blend_mode)
            {
                let (meshes, particles) = blend_group.into_iter().fold(
                    (vec![], vec![]),
                    |mut acc, pass_data| {
                        match pass_data.data {
                            DrawData::Meshes(mesh_draw) => {
                                acc.0.push(MeshDrawData {
                                    blend_mode,
                                    texture: pass_data.texture,
                                    data: mesh_draw,
                                })
                            }
                            DrawData::Particles(particle_draw) => {
                                acc.1.push(ParticleDrawData {
                                    blend_mode,
                                    texture: pass_data.texture,
                                    data: particle_draw,
                                })
                            }
                        }

                        acc
                    },
                );


                for ((blend_mode, texture), mesh_group) in &meshes
                    .into_iter()
                    .sorted_by_key(|x| x.texture)
                    .group_by(|x| (x.blend_mode, x.texture))
                {
                    self.render_meshes(
                        is_first,
                        params.clear_color,
                        MeshDrawData {
                            blend_mode,
                            texture,
                            data: mesh_group
                                .flat_map(|x| x.data)
                                .collect_vec()
                                .into(),
                        },
                        &surface_view,
                    );

                    perf_counter_inc("real_mesh_draw", 1);
                    is_first = false;
                }

                for ((blend_mode, texture), particle_group) in &particles
                    .into_iter()
                    .group_by(|x| (x.blend_mode, x.texture))
                {
                    self.render_particles(
                        is_first,
                        ParticleDrawData {
                            blend_mode,
                            texture,
                            data: particle_group
                                .flat_map(|x| x.data)
                                .collect_vec(),
                        },
                        params.clear_color,
                        &surface_view,
                    );

                    perf_counter_inc("real_particle_draw", 1);
                    is_first = false;
                }


                // match pass_data.data {
                //     DrawData::Mesh(_) => {}
                //     DrawData::Particle(_) => {
                //     }
                // }

                is_first = false;
            }
        }

        self.render_post_processing(&surface_view, params.lighting);
        self.render_egui(&surface_view);

        if params.config.dev.show_buffers {
            self.render_debug(&surface_view);
        }

        #[cfg(feature = "record-pngs")]
        {
            {
                let mut encoder = self.context.device.create_command_encoder(
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
                        buffer: &self.screenshot_buffer.buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(
                                std::mem::size_of::<u32>() as u32 *
                                    self.config.width,
                            ),
                            rows_per_image: Some(self.config.height),
                        },
                    },
                    output.texture.size(),
                );

                self.context.queue.submit(std::iter::once(encoder.finish()));
            }

            pollster::block_on(async {
                let buffer_slice = self.screenshot_buffer.buffer.slice(..);

                let (tx, rx) =
                    futures_intrusive::channel::shared::oneshot_channel();

                buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                    tx.send(result).unwrap();
                });

                self.context.device.poll(wgpu::Maintain::Wait);
                rx.receive().await.unwrap().unwrap();

                let data = buffer_slice.get_mapped_range();

                let path = std::env::current_exe().unwrap();
                let example_name = path.file_name().unwrap().to_string_lossy();

                let images_dir = format!("target/images/{}", example_name);

                let videos_dir = "target/videos";
                let screenshots_dir = "target/screenshots";

                std::fs::create_dir_all(&images_dir).unwrap();
                std::fs::create_dir_all(&videos_dir).unwrap();
                std::fs::create_dir_all(&screenshots_dir).unwrap();

                let name =
                    format!("{}/image-{:03}.png", &images_dir, get_frame());

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


                    let mut buffer =
                        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                            self.config.width,
                            self.config.height,
                            rgba_data,
                        )
                        .unwrap();

                    let resized = image::imageops::resize(
                        &buffer,
                        self.config.width / 3,
                        self.config.height / 3,
                        image::imageops::FilterType::Nearest,
                    );

                    resized.save(name).unwrap();

                    if get_frame() > 60 {
                        resized.save(format!("{}/{}.png", screenshots_dir, example_name)).unwrap();

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

            self.screenshot_buffer.buffer.unmap();
        }

        output.present();
    }

    pub fn scale_factor(&self) -> f32 {
        // self.window.scale_factor() as f32
        1.0
    }

    pub fn resize(&mut self, new_size: UVec2) {
        let _span = span!("resize");

        let scale_factor = self.window.scale_factor() as f32;

        let new_size =
            winit::dpi::PhysicalSize::<u32>::new(new_size.x, new_size.y);

        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.context.device, &self.config);
        }

        self.egui_render_routine.borrow_mut().resize(
            new_size.width,
            new_size.height,
            scale_factor,
        );

        self.egui_winit.set_pixels_per_point(scale_factor);
    }

    pub fn egui_ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }

    pub fn load_textures(&mut self, texture_queue: TextureLoadQueue) {
        for item in texture_queue {
            self.tx_texture.send(item).log_err();
        }

        // let _span = span!("load_textures");
        // // #[cfg(not(target_arch = "wasm32"))]
        // // let iter = texture_queue.into_par_iter();
        // // #[cfg(target_arch = "wasm32")]
        // // let iter = texture_queue.into_iter();
        // let iter = texture_queue.into_par_iter();
        //
        // let results: Vec<_> = iter
        //     .filter_map(|item| {
        //         info!("Loading image: {}", item.path);
        //
        //         let texture = Texture::from_image(
        //             &self.device,
        //             &self.queue,
        //             &item.image,
        //             Some(&item.path),
        //             false,
        //         )
        //         .ok()?;
        //
        //         let bind_group = self.device.simple_bind_group(
        //             &format!("{}_bind_group", item.path),
        //             &texture,
        //             &self.texture_bind_group_layout,
        //         );
        //
        //         Some((item.handle, texture, bind_group))
        //     })
        //     .collect();
        //
        // for (handle, texture, bind_group) in results.into_iter() {
        //     self.textures.lock().insert(handle, (bind_group, texture));
        // }
    }

    pub fn width(&self) -> f32 {
        self.config.width as f32
    }

    pub fn height(&self) -> f32 {
        self.config.height as f32
    }

    pub fn end_frame(&mut self) {}
}

fn depth_stencil_attachment(
    enabled: bool,
    view: &wgpu::TextureView,
    is_first: bool,
) -> Option<wgpu::RenderPassDepthStencilAttachment> {
    let clear_depth =
        if is_first { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load };

    if enabled {
        Some(wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(wgpu::Operations {
                load: clear_depth,
                store: true,
            }),
            stencil_ops: None,
        })
    } else {
        None
    }
}

use gif::{Encoder, Frame, Repeat};

pub fn save_gif(
    path: &str,
    frames: &mut Vec<Vec<u8>>,
    speed: i32,
    width: u16,
    height: u16,
) -> Result<()> {
    use image::{ImageBuffer, Rgba};

    let out_w = width as u32 / 8;
    let out_h = height as u32 / 8;

    let mut image_file = std::fs::File::create(path)?;
    let mut encoder =
        Encoder::new(&mut image_file, out_w as u16, out_h as u16, &[
            0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255,
        ])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for mut frame in frames {
        let image = ImageBuffer::<Rgba<u8>, _>::from_raw(
            width as u32,
            height as u32,
            frame.as_slice(),
        )
        .unwrap();

        let resized = image::imageops::resize(
            &image,
            out_w,
            out_h,
            image::imageops::FilterType::Nearest,
        );

        let mut resized = resized.into_raw();

        encoder.write_frame(&Frame::from_rgba_speed(
            out_w as u16,
            out_h as u16,
            &mut resized,
            speed,
        ))?;
    }

    Ok(())
}
