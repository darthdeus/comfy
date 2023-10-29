use std::sync::mpsc::{channel, Receiver, Sender};

use crate::*;

use image::Rgba;
use winit::window::Window;

pub type PipelineMap = HashMap<String, wgpu::RenderPipeline>;
pub type TextureMap = HashMap<TextureHandle, (wgpu::BindGroup, Texture)>;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadUniform {
    pub clip_position: [f32; 2],
    pub size: [f32; 2],
}

pub fn shader_to_wgpu(shader: &Shader) -> wgpu::ShaderModuleDescriptor<'_> {
    wgpu::ShaderModuleDescriptor {
        label: Some(&shader.name),
        source: wgpu::ShaderSource::Wgsl(shader.source.as_str().into()),
    }
}

#[derive(Clone)]
pub struct GraphicsContext {
    pub surface: Arc<wgpu::Surface>,
    pub instance: Arc<wgpu::Instance>,
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    // Shared for all regular textures/sprites
    pub texture_layout: Arc<wgpu::BindGroupLayout>,
    pub config: Arc<AtomicRefCell<wgpu::SurfaceConfiguration>>,
}

pub struct WgpuRenderer {
    pub context: GraphicsContext,

    #[cfg(not(any(feature = "ci-release", target_arch = "wasm32")))]
    pub hot_reload: HotReload,

    pub pipelines: PipelineMap,
    pub shaders: RefCell<ShaderMap>,

    pub egui_winit: egui_winit::State,
    pub egui_render_routine: RefCell<EguiRenderRoutine>,

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
    pub loaded_image_recv: Receiver<LoadedImage>,
    pub loaded_image_send: Sender<LoadedImage>,

    pub textures: Arc<Mutex<TextureMap>>,
}

impl WgpuRenderer {
    pub async fn new(window: Window, egui_winit: egui_winit::State) -> Self {
        let context = create_graphics_context(&window).await;

        trace!("Loading builtin engine textures");

        let mut textures = HashMap::default();

        macro_rules! load_engine_tex {
            ($name: literal) => {{
                load_texture_from_engine_bytes(
                    &context,
                    $name,
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/assets/",
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
        load_engine_tex!("_builtin-comfy");

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

        let (width, height, format) = {
            let config = context.config.borrow();
            (config.width, config.height, config.format)
        };

        let egui_render_routine = EguiRenderRoutine::new(
            &context.device,
            format,
            1,
            width,
            height,
            window.scale_factor() as f32,
        );

        let screenshot_buffer = SizedBuffer::new(
            "screenshot_buffer",
            &context.device,
            (width * height) as usize * std::mem::size_of::<u32>(),
            BufferType::Read,
        );

        info!("Initializing with scale factor: {}", window.scale_factor());

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

        let first_pass_texture = Texture::create_scaled_surface_texture(
            &context.device,
            &context.config.borrow(),
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
            &context.config.borrow(),
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

        let render_texture_format = wgpu::TextureFormat::Rgba16Float;

        let bloom = Bloom::new(
            &context,
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
            &context.config.borrow(),
            "Depth Texture",
        );

        Self {
            #[cfg(not(target_arch = "wasm32"))]
            thread_pool: rayon::ThreadPoolBuilder::new().build().unwrap(),

            loaded_image_recv: rx_texture,
            loaded_image_send: tx_texture,

            depth_texture: Arc::new(depth_texture),

            pipelines: HashMap::new(),
            shaders: RefCell::new(HashMap::new()),
            #[cfg(not(any(feature = "ci-release", target_arch = "wasm32")))]
            hot_reload: HotReload::new(),

            screenshot_buffer,

            vertex_buffer,
            index_buffer,

            post_processing_effects: RefCell::new(Vec::new()),
            bloom,

            egui_winit,
            egui_render_routine: RefCell::new(egui_render_routine),

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
        game_config: &GameConfig,
    ) {
        let _span = span!("render_post_processing");
        let mut encoder =
            self.context.device.simple_encoder("Post Processing Encoder");

        let mut input_bind_group = &self.first_pass_bind_group;

        if game_config.bloom_enabled {
            self.bloom.draw(
                &self.context.device,
                &self.texture_layout,
                &self.first_pass_bind_group,
                &mut encoder,
            );
        }

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

                if let Some(shader) = self.shaders.borrow().get(&effect.id) {
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
                    warn!(
                        "NO SHADER FOR EFFECT: {} ... {}",
                        effect.name, effect.id
                    );
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
                error!("Missing pipeline for {}", effect.name);
            }
        }

        if game_config.bloom_enabled {
            self.bloom.blit_final(
                &mut encoder,
                &self.tonemapping_texture.view,
                &game_config.lighting,
            );
        }

        let tonemapping_pipeline =
            self.pipelines.entry("tonemapping".into()).or_insert_with(|| {
                create_post_processing_pipeline(
                    "Tonemapping",
                    &self.context.device,
                    self.context.config.borrow().format,
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
                    self.context.config.borrow().format,
                    &[&self.texture_layout, &self.quad_ubg.layout],
                    &[],
                    // TODO: .shaders.get_or_err(...)
                    &reloadable_wgsl_shader!("debug"),
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

    pub fn render_egui(&self, view: &wgpu::TextureView, params: &DrawParams) {
        let _span = span!("render_egui");

        let mut encoder =
            self.context.device.simple_encoder("egui Render Encoder");

        let paint_jobs =
            self.egui_render_routine.borrow_mut().end_frame_and_render(
                params.egui,
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

    pub fn on_event(
        &mut self,
        event: &winit::event::WindowEvent,
        egui_ctx: &egui::Context,
    ) -> bool {
        self.egui_winit.on_event(egui_ctx, event).consumed
    }

    pub fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    pub fn begin_frame(&mut self, egui_ctx: &egui::Context) {
        let _span = span!("begin_frame");

        egui_ctx.begin_frame(self.egui_winit.take_egui_input(&self.window));
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
            let _span = span!("wgpu texture load");

            while let Ok(loaded_image) = self.loaded_image_recv.try_recv() {
                let context = self.context.clone();
                let textures = self.textures.clone();
                let tbgl = self.texture_layout.clone();

                let load_image_texture = move || {
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
                load_image_texture();

                #[cfg(not(target_arch = "wasm32"))]
                self.thread_pool.spawn(load_image_texture);
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

        // TODO: re-implement shader reloading properly
        #[cfg(not(any(feature = "ci-release", target_arch = "wasm32")))]
        if self.hot_reload.maybe_reload_shaders() {
            panic!("TODO: shader hot reloading is currently unsupported")
            //     // TODO: this breaks previously loaded user shaders
            //     error!("TODO: reload breaks previously loaded user shaders");
            //     self.shaders = RefCell::new(load_shaders());
            //     self.pipelines.clear();
        }

        self.camera_uniform.update_view_proj(&main_camera());

        params.config.lighting.time = get_time() as f32;
        {
            let camera = main_camera();
            params.config.lighting.chromatic_aberration =
                (camera.shake_amount * 200.0 * camera.shake_timer)
                    .clamp(0.0, 200.0);
        }

        self.context.queue.write_buffer(
            &self.global_lighting_params_buffer,
            0,
            bytemuck::cast_slice(&[params.config.lighting]),
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

            match self.context.surface.get_current_texture() {
                Ok(texture) => texture,
                Err(_) => {
                    return;
                }
            }
        };

        let surface_view = {
            let _span = span!("create surface view");
            output.texture.create_view(&wgpu::TextureViewDescriptor::default())
        };

        draw_batched_render_passes(self, &surface_view, &params);

        self.render_post_processing(&surface_view, params.config);
        self.render_egui(&surface_view, &params);

        if params.config.dev.show_buffers {
            self.render_debug(&surface_view);
        }

        #[cfg(feature = "record-pngs")]
        screenshot::record_pngs(
            uvec2(self.config.width, self.config.height),
            &self.context,
            &self.screenshot_buffer,
            &output,
        );

        output.present();
    }

    pub fn scale_factor(&self) -> f32 {
        // self.window.scale_factor() as f32
        1.0
    }

    pub fn resize(&mut self, new_size: UVec2) {
        let _span = span!("resize");

        let scale_factor = self.window.scale_factor() as f32;

        let size = winit::dpi::PhysicalSize::<u32>::new(new_size.x, new_size.y);

        {
            let mut config = self.context.config.borrow_mut();

            config.width = size.width;
            config.height = size.height;

            self.context.surface.configure(&self.context.device, &config);
        }

        self.egui_render_routine.borrow_mut().resize(
            size.width,
            size.height,
            scale_factor,
        );

        self.egui_winit.set_pixels_per_point(scale_factor);
    }

    pub fn width(&self) -> f32 {
        self.context.config.borrow().width as f32
    }

    pub fn height(&self) -> f32 {
        self.context.config.borrow().height as f32
    }

    pub fn end_frame(&mut self) {}
}

pub fn depth_stencil_attachment(
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
