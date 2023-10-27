use std::num::NonZeroU32;

use crate::prelude::*;

pub struct GlowRenderer {
    gl: Arc<glow::Context>,
    window: Window,
    egui_integration: EguiIntegration,
    resolution: UVec2,
    textures: Arc<AtomicRefCell<HashMap<TextureHandle, Texture>>>,

    light_ubo: glow::NativeBuffer,
    frame_data_ubo: glow::NativeBuffer,

    batch: StaticBatch,

    #[allow(dead_code)]
    particle_shader: Shader,

    first_pass: FrameBuffer,
    bloom: Bloom,
    color_correction: PostProcessing,
    #[allow(dead_code)]
    color_replacement: PostProcessing,
    chromatic_aberration: PostProcessing,
    screen_shake: PostProcessing,

    #[allow(dead_code)]
    shaders: Vec<Shader>,
}

// impl Renderer for GlowRenderer {
impl GlowRenderer {
    fn begin_frame(&mut self) {
        self.egui_integration.begin_frame(&self.window);
    }

    fn end_frame(&mut self) {
        self.egui_integration.end_frame(&self.window);
    }

    fn update(&mut self, _params: &mut DrawParams) {
        if (is_key_down(KeyCode::LAlt) || is_key_down(KeyCode::RAlt)) &&
            is_key_pressed(KeyCode::Return)
        {
            match self.window.fullscreen_state() {
                sdl2::video::FullscreenType::Off => {
                    self.window
                        .set_fullscreen(sdl2::video::FullscreenType::Desktop)
                        .log_err();
                }
                sdl2::video::FullscreenType::True => {
                    self.window
                        .set_fullscreen(sdl2::video::FullscreenType::Off)
                        .log_err();
                }
                sdl2::video::FullscreenType::Desktop => {
                    self.window
                        .set_fullscreen(sdl2::video::FullscreenType::Off)
                        .log_err();
                }
            }
        }
    }

    fn draw(&mut self, mut params: DrawParams) {
        let _span = span!("draw");

        let (width, height) = self.window.drawable_size();

        let mut shader_flags = HashMap::default();

        shader_flags.insert("skip_pp".to_string(), false);

        let pp_params = PostProcessingParams {
            flags: &shader_flags,
            frame: params.frame.frame,
            delta: params.frame.delta,
            time: params.frame.time,
            resolution: self.resolution,
        };

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);

            // self.draw_scene(frame_params.clear_color, draw_queue,
            // mesh_queue);
            //
            {
                let _span = gl_span!(&self.gl, "first-pass");

                // self.gl.push_safe_group(
                //     glow::DEBUG_SOURCE_APPLICATION,
                //     0,
                //     "first-pass",
                // );

                self.first_pass.bind();
                self.draw_scene(&mut params);
                self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);

                // self.gl.pop_safe_group();
            }

            {
                let _span = gl_span!(&self.gl, "bloom");

                self.bloom.draw_threshold(self.first_pass.color_buffer);
                self.bloom.draw_blur();

                self.color_correction.bind_begin();
                self.bloom.draw_blend(self.first_pass.color_buffer);
                self.color_correction.bind_end();

                // self.color_replacement.bind_begin();
                // draw_quad(&self.gl);
                // self.color_replacement.bind_end();
            }

            // self.color_correction.bind_begin();
            // self.color_replacement.draw(&params);
            // self.color_correction.bind_end();

            {
                let _span = gl_span!(&self.gl, "color-correction");

                self.chromatic_aberration.bind_begin();
                self.color_correction.draw(&pp_params);
                self.chromatic_aberration.bind_end();
            }

            {
                let _span = gl_span!(&self.gl, "screen-shake");

                self.screen_shake.bind_begin();
                self.chromatic_aberration.draw(&pp_params);
                self.screen_shake.bind_end();
            }

            self.screen_shake.draw(&pp_params);

            // self.post_processing[0].bind_begin();
            // self.draw_scene(frame_params.clear_color, draw_queue,
            // mesh_queue); self.post_processing[0].bind_end();
            //
            // let max = self.post_processing.len();
            //
            // for i in 1..max {
            //     if i < max - 1 {
            //         self.post_processing[i].bind_begin();
            //     }
            //
            //     self.post_processing[i - 1].render(&params);
            //
            //     if i < max - 1 {
            //         self.post_processing[i].bind_end();
            //     }
            // }

            {
                let _span = gl_span!(&self.gl, "egui");
                self.egui_integration.paint(uvec2(width, height));
            }

            {
                let _span = gl_span!(&self.gl, "swap");
                self.window.gl_swap_window();
            }
        }
    }

    fn scale_factor(&self) -> f32 {
        // TODO
        1.0
        // self.window.scale_factor() as f32
    }

    fn resize(&mut self, new_size: UVec2) {
        let _span = span!("resize");

        self.resolution = new_size;

        {
            let mut state = GLOBAL_STATE.borrow_mut();
            state.screen_size = new_size.as_vec2();
            main_camera_mut().aspect_ratio =
                new_size.x as f32 / new_size.y as f32;
        }

        unsafe {
            let new_size = new_size.as_ivec2();

            self.first_pass.resize(&self.gl, new_size);

            self.bloom.resize(new_size);
            self.color_correction.resize(new_size);
            self.color_replacement.resize(new_size);
            self.chromatic_aberration.resize(new_size);
            self.screen_shake.resize(new_size);
        }

        // let _new_size =
        //     winit::dpi::PhysicalSize::<u32>::new(new_size.x, new_size.y);

        // if new_size.width > 0 && new_size.height > 0 {
        //     self.size = new_size;
        //     self.config.width = new_size.width;
        //     self.config.height = new_size.height;
        //     self.surface.configure(&self.device, &self.config);
        // }

        // self.egui_render_routine.borrow_mut().resize(
        //     new_size.width,
        //     new_size.height,
        //     scale_factor,
        // );

        // self.egui_winit.set_pixels_per_point(scale_factor);
    }

    fn egui_ctx(&self) -> &egui::Context {
        &self.egui_integration.ctx
    }

    fn load_textures(&mut self, texture_queue: TextureLoadQueue) {
        let _span = span!("load_textures");

        for loaded_image in texture_queue.into_iter() {
            let image = loaded_image.image.flipv();

            let texture = Texture::from_rgba_bytes(
                &loaded_image.path,
                self.gl.clone(),
                &image.to_rgba8(),
                image.width(),
                image.height(),
            );

            self.textures.borrow_mut().insert(loaded_image.handle, texture);
        }
    }

    fn width(&self) -> f32 {
        self.resolution.x as f32
    }

    fn height(&self) -> f32 {
        self.resolution.y as f32
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

enum GlDebugSeverity {
    Notification,
    Low,
    Medium,
    High,
}

impl GlowRenderer {
    pub fn new(
        gl: Arc<glow::Context>,
        window: Window,
        // window: winit::window::Window,
        // egui_winit: egui_winit::State,
    ) -> Self {
        let resolution = {
            let (width, height) = window.size();
            ivec2(width as i32, height as i32)
        };

        unsafe {
            init_quad_buffers(gl.clone());
        }

        #[cfg(target_os = "linux")]
        unsafe {
            if false {
                gl.debug_message_control(
                    glow::DONT_CARE,
                    glow::DONT_CARE,
                    glow::DONT_CARE,
                    &[],
                    true,
                );
            }

            gl.debug_message_callback(
                |_source, typ, _id, severity, message| {
                    let typ = match typ {
                        glow::DEBUG_TYPE_ERROR => "ERROR",
                        glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => {
                            "DEPRECATED_BEHAVIOR"
                        }
                        glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => {
                            "UNDEFINED_BEHAVIOR"
                        }
                        glow::DEBUG_TYPE_PORTABILITY => "PORTABILITY",
                        glow::DEBUG_TYPE_PERFORMANCE => {
                            // "PERFORMANCE",
                            debug!("Ignoring GL perf warning");
                            return;
                        }
                        glow::DEBUG_TYPE_OTHER => "OTHER",
                        _ => "!!!!! UNKNOWN TYPE !!!!!",
                    };

                    let severity = match severity {
                        glow::DEBUG_SEVERITY_LOW => GlDebugSeverity::Low,
                        glow::DEBUG_SEVERITY_MEDIUM => GlDebugSeverity::Medium,
                        glow::DEBUG_SEVERITY_HIGH => GlDebugSeverity::High,
                        glow::DEBUG_SEVERITY_NOTIFICATION => {
                            GlDebugSeverity::Notification
                        }
                        _ => {
                            error!("Unexpected GlDebugSeverity: {severity}");
                            GlDebugSeverity::High
                        }
                    };

                    match severity {
                        GlDebugSeverity::Notification => {
                            // info!("[GL][{typ}]: {message}");
                        }
                        GlDebugSeverity::Low => {
                            warn!("[GL][{typ}]: {message}");
                        }
                        GlDebugSeverity::Medium => {
                            error!("[GL][{typ}]: {message}");
                        }
                        GlDebugSeverity::High => {
                            error!("[GL CRIT][{typ}]: {message}");
                        }
                    }
                },
            );
        }


        let egui_integration = EguiIntegration::new(gl.clone());

        let mut fonts = egui::FontDefinitions::default();

        // // Install my own font (maybe supporting non-latin characters):
        // fonts.font_data.insert(
        //     "fira".to_owned(),
        //     egui::FontData::from_static(include_bytes!(
        //         "../../assets/FiraMono-Medium.ttf"
        //     )),
        // ); // .ttf and .otf supported

        // Put my font first (highest priority):
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "fira".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .push("fira".to_owned());

        egui_integration.ctx.set_fonts(fonts);

        let (width, height) = window.size();

        let shader = Shader::new(
            "main",
            gl.clone(),
            reloadable_str!("shaders/vert.glsl"),
            reloadable_str!("shaders/frag.glsl"),
        );

        let batch = unsafe { StaticBatch::new(gl.clone(), shader) };

        let light_ubo = unsafe {
            let light_ubo =
                gl.create_buffer().expect("failed to create light_ubo");

            gl.bind_buffer(glow::UNIFORM_BUFFER, Some(light_ubo));
            gl.safe_label(glow::BUFFER, light_ubo.0.into(), Some("Light UBO"));

            gl.buffer_data_size(
                glow::UNIFORM_BUFFER,
                std::mem::size_of::<LightUniform>() as i32,
                glow::STATIC_DRAW,
            );
            gl.bind_buffer(glow::UNIFORM_BUFFER, None);

            light_ubo
        };

        let frame_data_ubo = unsafe {
            let frame_data_ubo =
                gl.create_buffer().expect("failed to create frame_data_ubo");

            gl.bind_buffer(glow::UNIFORM_BUFFER, Some(frame_data_ubo));
            gl.safe_label(
                glow::BUFFER,
                frame_data_ubo.0.into(),
                Some("Frame UBO"),
            );

            gl.buffer_data_size(
                glow::UNIFORM_BUFFER,
                std::mem::size_of::<FrameDataUniform>() as i32,
                glow::STATIC_DRAW,
            );
            gl.bind_buffer(glow::UNIFORM_BUFFER, None);

            frame_data_ubo
        };

        let mut textures = HashMap::default();

        // TODO: unify this with wgpu renderer
        {
            {
                let white_tex_handle = TextureHandle::from_path("1px");
                let white_tex = Texture::new(
                    "1px",
                    gl.clone(),
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/assets/1px.png"
                    )),
                );

                ASSETS.borrow_mut().insert_handle("1px", white_tex_handle);
                textures.insert(white_tex_handle, white_tex);
            }

            {
                let error_tex_handle = TextureHandle::from_path("error");
                let error_tex = Texture::new(
                    "error",
                    gl.clone(),
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/assets/error.png"
                    )),
                );

                ASSETS.borrow_mut().insert_handle("error", error_tex_handle);
                textures.insert(error_tex_handle, error_tex);
            }
        }

        let textures = Arc::new(AtomicRefCell::new(textures));

        BLOOD_CANVAS
            .set(AtomicRefCell::new(BloodCanvas::new(Arc::new(
                AtomicRefCell::new(GlowTextureCreator {
                    gl: gl.clone(),
                    textures: textures.clone(),
                }),
            ))))
            .expect("failed to create glow blood canvas");


        Self {
            window,
            gl: gl.clone(),
            egui_integration,

            light_ubo,
            frame_data_ubo,

            batch,

            particle_shader: Shader::new(
                "particles",
                gl.clone(),
                reloadable_str!("shaders/particles.vert"),
                reloadable_str!("shaders/particles.frag"),
            ),

            // size,
            // egui_winit,
            resolution: uvec2(width, height),
            textures,

            first_pass: FrameBuffer::new("first-pass", resolution, gl.clone()),
            bloom: Bloom::new(resolution, gl.clone()),

            shaders: vec![
                // Shader::new(
                //     "space-bg",
                //     gl.clone(),
                //     reloadable_str!("shaders/vert.glsl"),
                //     reloadable_str!("shaders/space-bg.frag"),
                // ),
                Shader::new(
                    "simple-quad",
                    gl.clone(),
                    reloadable_str!("shaders/vert.glsl"),
                    reloadable_str!("shaders/test-bg.frag"),
                ),
            ],

            color_correction: PostProcessing::new(
                "color-grading",
                Shader::new(
                    "color-grading",
                    gl.clone(),
                    reloadable_str!("shaders/simple.vert"),
                    reloadable_str!("shaders/color-grading.frag"),
                ),
                resolution,
                &gl,
            ),

            color_replacement: PostProcessing::new(
                "color-replacement",
                Shader::new(
                    "color-replacement",
                    gl.clone(),
                    reloadable_str!("shaders/simple.vert"),
                    reloadable_str!("shaders/color-replacement.frag"),
                ),
                resolution,
                &gl,
            ),

            chromatic_aberration: PostProcessing::new(
                "chromatic-aberration",
                Shader::new(
                    "chromatic-aberration",
                    gl.clone(),
                    reloadable_str!("shaders/simple.vert"),
                    reloadable_str!("shaders/chromatic-aberration.frag"),
                ),
                resolution,
                &gl,
            ),

            screen_shake: PostProcessing::new(
                "screen-shake",
                Shader::new(
                    "screen-shake",
                    gl.clone(),
                    reloadable_str!("shaders/shake.vert"),
                    reloadable_str!("shaders/shake.frag"),
                ),
                resolution,
                &gl,
            ),
        }
    }

    pub fn on_event(&mut self, event: Event) -> bool {
        self.egui_integration.process_event(&self.window, event)
    }

    unsafe fn draw_scene(&self, params: &mut DrawParams) {
        let _span = span!("draw_scene");

        let (width, height) = self.window.drawable_size();

        self.gl.viewport(0, 0, width as i32, height as i32);
        self.gl.disable(glow::SCISSOR_TEST);
        self.gl.disable(glow::CULL_FACE);
        self.gl.disable(glow::FRAMEBUFFER_SRGB);

        self.gl.enable(glow::DEPTH_TEST);
        self.gl.depth_func(glow::LESS);
        self.gl.disable(glow::BLEND);

        // self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear_color(
            params.clear_color.r,
            params.clear_color.g,
            params.clear_color.b,
            // TODO: 1.0?
            params.clear_color.a,
            // 1.0,
        );

        self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

        self.gl.bind_vertex_array(Some(self.batch.vao));

        let mut light_uniform = LightUniform::default();

        for (i, light) in params.lights.iter().enumerate() {
            if i >= 128 {
                break;
            }

            light_uniform.lights[i] = *light;
            light_uniform.num_lights += 1;
        }

        self.gl.bind_buffer(glow::UNIFORM_BUFFER, Some(self.light_ubo));

        self.gl.buffer_data_u8_slice(
            glow::UNIFORM_BUFFER,
            bytemuck::cast_slice(&[light_uniform]),
            glow::STATIC_DRAW,
        );

        // let draw_queue = draw_queue.into_iter().map(
        //     |(texture, position, color, _, params)| {
        //         if let Some(texture) = self.textures.get(&texture) {
        //             texture.bind(&self.gl, &self.batch.shader, 0);
        //         } else {
        //             panic!("Missing texture :(");
        //         }
        //
        //         let size = params.dest_size.unwrap_or(Vec2::ONE);
        //
        //         let tex_width = size.x;
        //         let tex_height = size.y;
        //
        //         let vertices = rotated_rectangle(
        //             position, params, tex_width, tex_height, color,
        //         );
        //
        //         vertices
        //     },
        // );

        // self.batch
        //     .prepare(bytemuck::cast_slice(vertices), QUAD_INDICES_U32);
        // self.gl.draw_elements(
        //     glow::TRIANGLES,
        //     QUAD_INDICES_U32.len() as i32,
        //     glow::UNSIGNED_INT,
        //     0,
        // );

        let set_uniforms = |shader: &Shader| {
            shader.use_shader();

            shader.set_float_3(
                "iResolution",
                self.resolution.x as f32,
                self.resolution.y as f32,
                0.0,
            );

            shader.set_float_2("iMouse", 1.0, 1.0);
            shader.set_float("iTime", params.frame.time);
            shader.set_float("iTimeDelta", params.frame.delta);
            shader.set_int("iFrame", params.frame.frame as i32);
            shader.set_float("iFrameRate", 1.0 / params.frame.delta);

            shader.use_global("space_iterations");
            shader.use_global("space_formuparam");
            shader.use_global("space_volsteps");
            shader.use_global("space_stepsize");

            shader.use_global("space_zoom");
            shader.use_global("space_tile");
            shader.use_global("space_speed");

            shader.use_global("space_brightness");
            shader.use_global("space_darkmatter");
            shader.use_global("space_distfading");
            shader.use_global("space_saturation");

            shader.set_float("aspect_ratio", params.aspect_ratio);
            shader.set_mat4("projection", params.projection);

            if let Some(frame_data_index) =
                self.gl.get_uniform_block_index(shader.program, "FrameData")
            {
                self.gl.uniform_block_binding(
                    shader.program,
                    frame_data_index,
                    0,
                );

                self.gl.bind_buffer_base(
                    glow::UNIFORM_BUFFER,
                    0,
                    Some(self.frame_data_ubo),
                );
            }

            if let Some(lights_index) =
                self.gl.get_uniform_block_index(shader.program, "Lights")
            {
                self.gl.uniform_block_binding(shader.program, lights_index, 1);

                self.gl.bind_buffer_base(
                    glow::UNIFORM_BUFFER,
                    1,
                    Some(self.light_ubo),
                );
            }
        };

        {
            let shader = &self.batch.shader;

            set_uniforms(shader);

            for shader in self.shaders.iter() {
                set_uniforms(shader);
            }

            shader.use_shader();
        }

        self.batch.bind();

        let draw_current =
            |texture: TextureHandle,
             vertex_buffer: &mut Vec<SpriteVertex>,
             index_buffer: &mut Vec<u32>| {
                let textures = self.textures.borrow();

                let texture = match texture {
                    TextureHandle::Path(_) => {
                        textures
                            .get(&texture)
                            .unwrap_or_else(|| {
                                textures.get(&texture_id("error")).unwrap()
                            })
                            .texture
                    }
                    TextureHandle::Raw(id) => {
                        glow::NativeTexture(NonZeroU32::new(id as u32).unwrap())
                    }
                };

                self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));

                self.batch.upload(
                    bytemuck::cast_slice(vertex_buffer.as_slice()),
                    index_buffer,
                );

                self.gl.draw_elements(
                    glow::TRIANGLES,
                    index_buffer.len() as i32,
                    glow::UNSIGNED_INT,
                    0,
                );

                vertex_buffer.clear();
                index_buffer.clear();
            };

        {
            let _span = gl_span!(&self.gl, "mesh-queue");

            for (blend_mode, group) in &params
                .mesh_queue
                .iter()
                .group_by(|draw| draw.texture_params.blend_mode)
            {
                self.set_gl_blend_mode(blend_mode);

                for ((tex_handle, tex_params), group) in &group
                    .into_iter()
                    .group_by(|draw| (draw.mesh.texture, &draw.texture_params))
                {
                    let tex_handle = tex_handle.unwrap_or(params.white_px);

                    let mut all_vertices = vec![];
                    let mut all_indices = vec![];

                    let shader = if tex_params.shader.is_some() {
                        &self.shaders[0]
                    } else {
                        &self.batch.shader
                    };

                    set_uniforms(shader);
                    shader.use_shader();

                    for draw in group.sorted_by_key(|draw| draw.mesh.z_index) {
                        let mut mesh = draw.mesh.clone();

                        all_indices.extend(
                            mesh.indices
                                .iter()
                                .cloned()
                                .map(|x| x + all_vertices.len() as u32),
                        );

                        all_vertices.extend(mesh.vertices.drain(..));
                    }

                    draw_current(
                        tex_handle,
                        &mut all_vertices,
                        &mut all_indices,
                    );
                }
            }
        }

        {
            let _span = gl_span!(&self.gl, "particle-queue");

            self.gl.disable(glow::DEPTH_TEST);
            self.gl.enable(glow::BLEND);

            if GlobalParams::flag("additive-blending") {
                self.gl.blend_func(glow::ONE, glow::ONE);
            } else {
                self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            }

            for (blend_mode, group) in
                &params.particle_queue.iter().group_by(|draw| draw.blend_mode)
            {
                self.set_gl_blend_mode(blend_mode);

                for (texture, group) in
                    &group.into_iter().group_by(|draw| draw.texture)
                {
                    let mut vertex_buffer = vec![];
                    let mut index_buffer = vec![];

                    for draw in group {
                        let size = draw.size;

                        let tex_size = ASSETS
                            .borrow()
                            .texture_image_map
                            .lock()
                            .get(&texture)
                            .map(|image| {
                                vec2(
                                    image.width() as f32,
                                    image.height() as f32,
                                )
                            })
                            .unwrap_or(Vec2::ONE);

                        let tex_width = tex_size.x;
                        let tex_height = tex_size.y;

                        let vertices = rotated_rectangle(
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
                            Vec2::ZERO,
                        );

                        let len = vertex_buffer.len() as u32;
                        index_buffer.extend_from_slice(&[
                            0 + len,
                            2 + len,
                            1 + len,
                            0 + len,
                            3 + len,
                            2 + len,
                        ]);

                        vertex_buffer.extend(vertices);
                    }

                    if params.particle_queue.len() > 0 {
                        draw_current(
                            texture,
                            &mut vertex_buffer,
                            &mut index_buffer,
                        );
                    }
                }
            }
        }

        self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
    }


    unsafe fn set_gl_blend_mode(&self, blend_mode: BlendMode) {
        match blend_mode {
            BlendMode::None => {
                self.gl.enable(glow::DEPTH_TEST);
                self.gl.disable(glow::BLEND);
            }
            BlendMode::Additive => {
                self.gl.blend_func(glow::ONE, glow::ONE);
                self.gl.disable(glow::DEPTH_TEST);
                self.gl.enable(glow::BLEND);
            }
            BlendMode::Alpha => {
                self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                self.gl.disable(glow::DEPTH_TEST);
                self.gl.enable(glow::BLEND);
            }
        }
    }
}
