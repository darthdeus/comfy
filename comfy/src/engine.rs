use comfy_wgpu::WgpuRenderer;

use crate::*;

pub trait GameLoop {
    fn performance_metrics(&self, _world: &mut World, _ui: &mut egui::Ui) {}

    fn early_update(&mut self, _c: &mut EngineContext) {}
    // fn update<'a>(&'a mut self, _c: &'a mut EngineContext<'a>) {}
    fn update(&mut self, _c: &mut EngineContext) {}
    fn late_update(&mut self, _c: &mut EngineContext) {}
}

pub type GameLoopBuilder =
    Box<dyn Fn(&mut EngineContext) -> Arc<Mutex<dyn GameLoop>>>;

pub struct EngineState {
    pub cached_loader: RefCell<CachedImageLoader>,

    pub draw: RefCell<Draw>,

    pub frame: u64,
    pub flags: RefCell<HashSet<String>>,

    pub dt_stats: MovingStats,
    pub fps_stats: MovingStats,

    pub renderer: Option<WgpuRenderer>,
    pub texture_creator: Option<Arc<AtomicRefCell<WgpuTextureCreator>>>,

    pub lighting: GlobalLightingParams,

    pub meta: AnyMap,

    pub world: Rc<RefCell<World>>,
    pub commands: RefCell<CommandBuffer>,

    pub config: RefCell<GameConfig>,

    pub cooldowns: RefCell<Cooldowns>,
    pub changes: RefCell<ChangeTracker>,
    pub notifications: RefCell<Notifications>,

    pub builder: Option<GameLoopBuilder>,
    pub game_loop: Option<Arc<Mutex<dyn GameLoop>>>,

    pub is_paused: RefCell<bool>,
    pub show_pause_menu: bool,
    pub quit_flag: bool,

    pub to_despawn: RefCell<Vec<Entity>>,
}

impl EngineState {
    // TODO: get rid of GameLoopBuilder and replace it with a library-like API
    pub fn new(config: GameConfig, builder: GameLoopBuilder) -> Self {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                // console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            } else {
                #[cfg(feature = "ci-release")]
                std::panic::set_hook(Box::new(|info| {
                    error!("Panic: {:?}", info);
                }));

                initialize_logger();
            }
        }

        srand(thread_rng().next_u64());
        set_main_camera_zoom(30.0);

        ASSETS.borrow_mut().load_sound_from_bytes(
            "error",
            include_bytes!("../../assets/error.ogg"),
            StaticSoundSettings::default(),
        );

        Self {
            cached_loader: RefCell::new(CachedImageLoader::new()),

            renderer: None,
            texture_creator: None,

            draw: RefCell::new(Draw::new()),

            dt_stats: MovingStats::new(20),
            fps_stats: MovingStats::new(100),

            frame: 0,
            flags: RefCell::new(HashSet::new()),

            meta: AnyMap::new(),

            lighting: config.lighting,

            world: Rc::new(RefCell::new(World::new())),
            commands: RefCell::new(CommandBuffer::new()),

            config: RefCell::new(config),

            cooldowns: RefCell::new(Cooldowns::new()),
            changes: RefCell::new(ChangeTracker::new()),
            notifications: RefCell::new(Notifications::new()),

            builder: Some(builder),
            game_loop: None,

            is_paused: RefCell::new(false),
            show_pause_menu: false,
            quit_flag: false,

            to_despawn: RefCell::new(vec![]),
        }
    }

    fn begin_frame(&mut self) {
        let _span = span!("begin_frame");
        self.renderer.as_mut().unwrap().begin_frame();
    }

    #[cfg_attr(feature = "exit-after-startup", allow(unreachable_code))]
    pub fn update(&mut self) {
        let builder =
            if self.game_loop.is_none() { self.builder.take() } else { None };

        let mut c = self.make_context();

        if let Some(builder) = builder {
            *c.game_loop = Some((builder)(&mut c));
        }

        run_update_stages(&mut c);

        self.frame += 1;
    }

    pub fn make_context(&mut self) -> EngineContext {
        let renderer = self.renderer.as_ref().unwrap();
        let egui = renderer.egui_ctx();
        let texture_creator = self.texture_creator.as_ref().unwrap();

        EngineContext {
            cached_loader: &self.cached_loader,
            graphics_context: &renderer.context,
            textures: &renderer.textures,
            surface_config: &renderer.config,
            render_texture_format: renderer.render_texture_format,

            delta: delta(),

            egui,
            egui_wants_mouse: egui.wants_pointer_input(),

            draw: &mut self.draw,
            frame: self.frame,

            dt_stats: &mut self.dt_stats,
            fps_stats: &mut self.fps_stats,

            mouse_world: mouse_world(),

            flags: &self.flags,
            lighting: &mut self.lighting,

            meta: &mut self.meta,

            world: &mut self.world,
            commands: &self.commands,

            config: &self.config,
            game_loop: &mut self.game_loop,

            cooldowns: &mut self.cooldowns,
            changes: &self.changes,
            notifications: &mut self.notifications,

            post_processing_effects: &renderer.post_processing_effects,
            shaders: &renderer.shaders,

            is_paused: &self.is_paused,
            show_pause_menu: &mut self.show_pause_menu,
            quit_flag: &mut self.quit_flag,

            to_despawn: &mut self.to_despawn,

            texture_creator,
        }
    }

    pub fn one_frame(&mut self, delta: f32) {
        self.begin_frame();

        set_delta(delta);

        {
            let mut state = GLOBAL_STATE.borrow_mut();

            set_time(get_time() + delta as f64);
            state.fps = (1.0 / delta) as i32;
            state.egui_scale_factor =
                self.renderer.as_ref().unwrap().egui_ctx.pixels_per_point();
        }

        LightingState::begin_frame();

        if is_key_pressed(KeyCode::F7) {
            let mut config = self.config.borrow_mut();

            config.dev.show_lighting_config = !config.dev.show_lighting_config;
            config.dev.show_buffers = !config.dev.show_buffers;
        }

        if is_key_pressed(KeyCode::F8) {
            let mut config = self.config.borrow_mut();

            config.dev.show_fps = !config.dev.show_fps;
        }

        {
            let _span = span!("text");

            let renderer = self.renderer.as_mut().unwrap();

            ASSETS.borrow_mut().process_load_queue();
            ASSETS.borrow_mut().process_sound_queue();

            if let Some(texture_queue) =
                ASSETS.borrow_mut().current_queue.lock().take()
            {
                renderer.load_textures(texture_queue);
            }

            let painter =
                renderer.egui_ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Background,
                    egui::Id::new("text-painter"),
                ));

            let text_queue =
                GLOBAL_STATE.borrow_mut().text_queue.drain(..).collect_vec();

            for text in text_queue {
                let align = match text.align {
                    TextAlign::TopLeft => egui::Align2::LEFT_TOP,
                    TextAlign::Center => egui::Align2::CENTER_CENTER,
                    TextAlign::TopRight => egui::Align2::RIGHT_TOP,
                    TextAlign::BottomLeft => egui::Align2::LEFT_BOTTOM,
                    TextAlign::BottomRight => egui::Align2::RIGHT_BOTTOM,
                };

                // TODO: maybe better way of doing this?
                let screen_pos =
                    text.position.as_world().to_screen() / egui_scale_factor();

                painter.text(
                    egui::pos2(screen_pos.x, screen_pos.y),
                    align,
                    text.text,
                    text.font,
                    text.color.egui(),
                );
            }

            let mut global_state = GLOBAL_STATE.borrow_mut();
            let mut camera = main_camera_mut();

            let width = renderer.width();
            let height = renderer.height();

            camera.aspect_ratio = width / height;
            global_state.screen_size = vec2(width, height);

            let viewport = camera.world_viewport();

            let flipped_mouse_pos = vec2(
                global_state.mouse_position.x,
                global_state.screen_size.y - global_state.mouse_position.y,
            );

            let normalized = flipped_mouse_pos / global_state.screen_size *
                viewport -
                viewport / 2.0;

            if !global_state.mouse_locked {
                global_state.mouse_world = normalized + camera.center;
            }

            // TODO: make this configurable
            // renderer
            //     .window()
            //     .set_cursor_visible(global_state.mouse_locked);
        }

        if is_key_pressed(KeyCode::F1) {
            let mut global_state = GLOBAL_STATE.borrow_mut();
            global_state.mouse_locked = !global_state.mouse_locked;
        }

        self.update();

        #[cfg(feature = "tracy")]
        tracy_client::Client::running()
            .expect("client must be running")
            .secondary_frame_mark(tracy_client::frame_name!("update"));

        {
            let _span = span!("blood_canvas");

            // TODO: this really doesn't belong here
            blood_canvas_update_and_draw(|key, block| {
                draw_sprite_ex(
                    block.handle,
                    (key.as_vec2() + splat(0.5)) *
                        blood_block_world_size() as f32,
                    WHITE,
                    Z_BLOOD_CANVAS,
                    DrawTextureParams {
                        dest_size: Some(
                            splat(blood_block_world_size() as f32)
                                .as_world_size(),
                        ),
                        blend_mode: BlendMode::Alpha,
                        ..Default::default()
                    },
                );
            });
        }

        let renderer = self.renderer.as_mut().unwrap();

        let mut mesh_queue =
            GLOBAL_STATE.borrow_mut().mesh_queue.drain(..).collect_vec();

        mesh_queue.sort_by_key(|x| x.mesh.z_index);

        let clear_color = GLOBAL_STATE.borrow_mut().clear_color;

        renderer.end_frame();

        let mut light = mouse_screen();
        light.y = screen_height() - light.y;

        let frame_params =
            FrameParams { frame: get_frame(), delta, time: get_time() as f32 };

        SINGLE_PARTICLES.borrow_mut().retain_mut(|particle| {
            particle.update(delta);
            particle.lifetime_current > 0.0
        });

        // TODO: keep the same vec between frames
        let mut all_particles = Vec::new();

        for (_, (player_t, _)) in
            self.world.borrow().query::<(&Transform, &PlayerTag)>().iter()
        {
            // TODO; check that there is only one?

            for (_, (transform, _)) in self
                .world
                .borrow()
                .query::<(&mut Transform, &FollowPlayer)>()
                .iter()
            {
                transform.position = player_t.position;
            }
        }

        #[cfg(not(feature = "ci-release"))]
        for (entity, (_, transform)) in self
            .world
            .borrow_mut()
            .query_mut::<(&AnimatedSpriteBuilder, Option<&Transform>)>()
        {
            error!(
                "AnimatedSpriteBuilder found in ECS (entity = {:?}), make \
                 sure to call .build()",
                entity
            );

            if let Some(transform) = transform {
                draw_circle(transform.position, 0.5, RED, 499);
                draw_text(
                    "AnimatedSpriteBuilder in ECS",
                    transform.position,
                    WHITE,
                    TextAlign::Center,
                );
            }
        }

        for (_, (transform, particle_system)) in self
            .world
            .borrow_mut()
            .query_mut::<(&Transform, &mut ParticleSystem)>()
        {
            particle_system.update(transform.position, delta);

            all_particles.extend(
                particle_system
                    .particles
                    .iter()
                    .filter(|p| p.lifetime_current > 0.0)
                    .cloned(),
            );
        }

        let particle_queue = SINGLE_PARTICLES
            .borrow_mut()
            .iter()
            .chain(all_particles.iter())
            .map(|p| {
                ParticleDraw {
                    position: (p.position + p.offset).extend(p.z_index as f32),
                    rotation: p.rotation,
                    texture: p.texture,
                    // color: p.color,
                    color: p.current_color(),
                    size: p.size * p.current_size(),
                    source_rect: p.source_rect,
                    blend_mode: p.blend_mode,
                }
            })
            .collect_vec();

        if self.config.borrow().dev.show_lighting_config {
            egui::Window::new("Lighting")
                .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
                .show(renderer.egui_ctx(), |ui| {
                    lighting_ui(&mut self.lighting, ui);
                });

            // egui::Window::new("Post Processing").show(&self.egui(), |ui| {
            //     for i in 0..self.post_processing_effects.len() {
            //         ui.horizontal(|ui| {
            //             ui.add_enabled_ui(
            //                 i < self.post_processing_effects.len() - 1,
            //                 |ui| {
            //                     if ui.button("down").clicked() {
            //                         self.post_processing_effects.swap(i, i + 1);
            //                     }
            //                 },
            //             );
            //
            //             ui.add_enabled_ui(i > 0, |ui| {
            //                 if ui.button("up").clicked() {
            //                     self.post_processing_effects.swap(i - 1, i);
            //                 }
            //             });
            //
            //             let effect = &mut self.post_processing_effects[i];
            //             ui.checkbox(
            //                 &mut effect.enabled,
            //                 format!("{}: {}", i, effect.name),
            //             );
            //         });
            //     }
            // });
        }

        let mut draw_params = DrawParams {
            aspect_ratio: aspect_ratio(),
            config: &mut self.config.borrow_mut(),
            projection: main_camera().build_view_projection_matrix(),
            white_px: texture_path("1px"),
            clear_color,
            frame: frame_params,
            lights: LightingState::take_lights(),
            lighting: &mut self.lighting,
            // sprite_queue,
            mesh_queue,
            particle_queue,
        };

        renderer.update(&mut draw_params);
        renderer.draw(draw_params);

        {
            let mut global_state = GLOBAL_STATE.borrow_mut();
            global_state.just_pressed.clear();
            global_state.just_released.clear();
            global_state.mouse_just_pressed.clear();
            global_state.mouse_just_released.clear();
            global_state.mouse_wheel = (0.0, 0.0);
        }
    }

    // TODO: this really needs a cleanup
    pub fn set_renderer(&mut self, renderer: WgpuRenderer) {
        self.texture_creator = Some(renderer.texture_creator.clone());
        self.renderer = Some(renderer);
    }

    // TODO: this really needs a cleanup
    pub fn renderer(&mut self) -> &mut WgpuRenderer {
        self.renderer.as_mut().expect("renderer must be initialized")
    }

    // TODO: this really needs a cleanup
    pub fn resize(&mut self, new_size: UVec2) {
        self.renderer.as_mut().unwrap().resize(new_size);
    }

    // TODO: this really needs a cleanup
    pub fn quit_flag(&mut self) -> bool {
        self.quit_flag
    }

    // TODO: this really needs a cleanup
    pub fn title(&self) -> String {
        // TODO: make this configurable
        format!("{} (COMFY ENGINE)", self.config.borrow().game_name)
    }
}
