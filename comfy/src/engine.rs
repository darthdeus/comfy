use comfy_wgpu::{RunGameLoop, WgpuRenderer};

use crate::*;

pub trait GameLoop {
    fn performance_metrics(&self, _world: &mut World, _ui: &mut egui::Ui) {}

    fn early_update(&mut self, _c: &mut EngineContext) {}
    fn update(&mut self, _c: &mut EngineContext) {}
    fn late_update(&mut self, _c: &mut EngineContext) {}
}

impl RunGameLoop for EngineState {
    fn one_frame(&mut self, delta: f32) {
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
                draw_texture_z_ex(
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

    fn set_renderer(&mut self, renderer: WgpuRenderer) {
        self.texture_creator = Some(renderer.texture_creator.clone());
        self.renderer = Some(renderer);
    }

    fn renderer(&mut self) -> &mut WgpuRenderer {
        self.renderer.as_mut().expect("renderer must be initialized")
    }

    fn resize(&mut self, new_size: UVec2) {
        self.renderer.as_mut().unwrap().resize(new_size);
    }

    fn quit_flag(&mut self) -> bool {
        self.quit_flag
    }

    fn title(&self) -> String {
        format!("{} (FLOAT)", self.config.borrow().game_name)
    }
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

        {
            let mut assets = ASSETS.borrow_mut();

            let handle = Sound::from_path("handle");
            let bytes = include_bytes!("../../assets/error.ogg");

            let data = StaticSoundData::from_cursor(
                std::io::Cursor::new(bytes),
                Default::default(),
            )
            .unwrap();

            assets.sound_ids.insert("error".to_string(), handle);
            assets.sounds.lock().insert(handle, data);
        }

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
        #[cfg(feature = "exit-after-startup")]
        std::process::exit(0);

        let _span = span!("game-state update");

        #[cfg(any(feature = "quick-exit", feature = "dev"))]
        if is_key_down(KeyCode::F1) && is_key_pressed(KeyCode::Escape) {
            println!("fast exit");
            std::process::exit(0);
        }

        AudioSystem::process_sounds();

        let builder =
            if self.game_loop.is_none() { self.builder.take() } else { None };

        let mut c = self.make_context();

        if let Some(builder) = builder {
            *c.game_loop = Some((builder)(&mut c));
        }

        if !*c.is_paused.borrow() {
            set_unpaused_time(get_unpaused_time() + c.delta as f64);
        }

        // TODO: not ideal
        clear_background(BLACK);

        c.early_update();

        if is_key_pressed(KeyCode::Escape) {
            if *c.is_paused.borrow() && *c.show_pause_menu {
                info!("Resuming");
                *c.is_paused.borrow_mut() = false;
                *c.show_pause_menu = false;
            } else if !*c.is_paused.borrow() && !*c.show_pause_menu {
                info!("Pausing");
                *c.is_paused.borrow_mut() = true;
                *c.show_pause_menu = true;
            } else {
                info!("Nothing");
            }
        }

        if is_key_pressed(KeyCode::F6) {
            // TODO: bake in some basic sfx into the engine
            play_sound("alarm-next-stage");
            GlobalParams::toggle_flag("debug");
        }

        if GlobalParams::flag("debug") {
            egui::Window::new("Parameters")
                .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(150.0, -80.0))
                .show(c.egui, |ui| {
                    let mut params = GLOBAL_PARAMS.borrow_mut();

                    let floats = [
                        "filter-cutoff",
                        "filter-resonance",
                        "colorScale",
                        "exposure",
                        "---",
                        "bloomThreshold",
                        "bloom-lerp",
                        "bloomGamma",
                        "---",
                        "brightness",
                        "saturation",
                        "contrast",
                        "---",
                        "gamma",
                        "---",
                        "chromatic_aberration",
                    ];

                    for name in floats.iter() {
                        if *name == "---" {
                            ui.separator();
                        } else {
                            ui.horizontal(|ui| {
                                let range =
                                    params.floats.get_mut(name).unwrap();
                                ui.label(*name);

                                ui.add(
                                    egui::DragValue::new(&mut range.value)
                                        .speed(range.speed)
                                        .clamp_range(range.min..=range.max),
                                );
                            });
                        }
                    }

                    ui.separator();

                    let ints =
                        ["bloom_alg", "physics_substeps", "tonemapping_alg"];

                    for name in ints.iter() {
                        if *name == "---" {
                            ui.separator();
                        } else {
                            ui.horizontal(|ui| {
                                let range = params.ints.get_mut(name).unwrap();
                                ui.label(*name);

                                ui.add(
                                    egui::DragValue::new(&mut range.value)
                                        .speed(0.1)
                                        .clamp_range(
                                            range.min..=(range.max - 1),
                                        ),
                                );
                            });
                        }
                    }
                });
        }

        let mut transforms = HashMap::new();

        for (entity, transform) in c.world_mut().query_mut::<&Transform>() {
            transforms.insert(entity, *transform);
        }

        for (_, transform) in c.world().query::<&mut Transform>().iter() {
            let parent = if let Some(parent) = transform.parent {
                transforms
                    .get(&parent)
                    .cloned()
                    .unwrap_or(Transform::position(Vec2::ZERO))
            } else {
                Transform::position(Vec2::ZERO)
            };

            let combined = transform.compose_with_parent(&parent);

            transform.abs_position = combined.position;
            transform.abs_rotation = combined.rotation;
            transform.abs_scale = combined.scale;
        }

        for (_, (transform, light)) in
            c.world_mut().query_mut::<(&Transform, &PointLight)>()
        {
            add_light(Light::simple(
                transform.position,
                light.radius * light.radius_mod,
                light.strength * light.strength_mod,
            ))
        }

        if !*c.is_paused.borrow() && !c.flags.borrow().contains(PAUSE_PHYSICS) {
            c.cooldowns.borrow_mut().tick(c.delta);
            c.notifications.borrow_mut().tick(c.delta);
        }

        timings_add_value("delta", delta());

        let mut call_queue = vec![];

        if !*c.is_paused.borrow() {
            for (entity, sprite) in
                c.world().query::<&mut AnimatedSprite>().iter()
            {
                if sprite.state.update_and_finished(c.delta) {
                    c.commands().despawn(entity);

                    // TODO: maybe not needed? replace with option
                    let mut temp: ContextFn = Box::new(|_| {});
                    std::mem::swap(&mut sprite.on_finished, &mut temp);

                    call_queue.push(temp);
                }
            }
        }

        for call in call_queue.drain(..) {
            call(&mut c);
        }

        {
            let _span = span!("context.update");
            c.update();
        }

        {
            let _span = span!("context.late_update");
            c.late_update();
        }

        {
            let _span = span!("trails");

            for (_, (trail, transform)) in
                c.world_mut().query_mut::<(&mut Trail, &Transform)>()
            {
                if !*c.is_paused.borrow() {
                    trail.update(transform.position, c.delta);
                }
                trail.draw_mesh();
            }
        }

        {
            let _span = span!("drawables");

            let mut temp_drawables = vec![];

            std::mem::swap(&mut temp_drawables, &mut c.draw_mut().drawables);

            temp_drawables.retain_mut(|drawable| {
                if let Some(ref mut time) = drawable.time {
                    if *time <= 0.0 {
                        false
                    } else {
                        *time -= c.delta;
                        (drawable.func)(&mut c);
                        true
                    }
                } else {
                    (drawable.func)(&mut c);
                    false
                }
            });

            let mut draw = c.draw_mut();
            std::mem::swap(&mut temp_drawables, &mut draw.drawables);

            for drawable in temp_drawables.drain(..) {
                draw.drawables.push(drawable);
            }
        }

        {
            let _span = span!("sprite_queue");

            let world = c.world.borrow();
            let mut sprite_query = world.query::<(&Sprite, &Transform)>();
            let sprite_iter = sprite_query
                .iter()
                .map(|(_, (sprite, transform))| sprite.to_quad_draw(transform));

            let mut animated_sprite_query =
                world.query::<(&AnimatedSprite, &Transform)>();

            let animated_sprite_iter = animated_sprite_query
                .iter()
                .map(|(_, (sprite, transform))| sprite.to_quad_draw(transform));

            for (z_index, group) in sprite_iter
                .chain(animated_sprite_iter)
                .group_by(|draw| draw.z_index)
                .into_iter()
                .sorted_by_key(|(z_index, _)| *z_index)
            {
                // for draw in group.sorted_by(|a, b| a.texture.cmp(&b.texture)) {
                for draw in group {
                    draw_texture_z_ex(
                        draw.texture,
                        draw.transform.position,
                        draw.color,
                        z_index,
                        DrawTextureParams {
                            source_rect: draw.source_rect,
                            dest_size: Some(draw.dest_size.as_world_size()),
                            rotation: draw.transform.rotation,
                            blend_mode: draw.blend_mode,
                            flip_x: draw.flip_x,
                            flip_y: draw.flip_y,
                            ..Default::default()
                        },
                    );
                }
            }
        }

        {
            let _span = span!("temp draws");

            for texture in c.draw_mut().textures.drain(..) {
                draw_texture_z_ex(
                    texture.texture,
                    texture.position.to_world(),
                    texture.color,
                    20, // TODO
                    texture.params,
                );
            }

            for (position, radius, color) in c.draw_mut().circles.drain(..) {
                draw_circle(position.to_world(), radius, color, 200);
            }

            for line in c.draw_mut().lines.drain(..) {
                draw_line(
                    line.start.to_world(),
                    line.end.to_world(),
                    line.width,
                    line.color,
                    70,
                );
            }

            // TODO: calculate world space font size
            for (text, position, color, size) in c.draw_mut().texts.drain(..) {
                draw_text_ex(&text, position, TextAlign::Center, TextParams {
                    color,
                    font: egui::FontId::new(size, egui::FontFamily::Monospace),
                    ..Default::default()
                });
            }
        }

        combat_text_system(&mut c);

        if !c.notifications.borrow_mut().notifications.is_empty() {
            egui::Window::new("Notifications")
                .anchor(egui::Align2::LEFT_TOP, egui::vec2(20.0, 280.0))
                .resizable(false)
                .title_bar(false)
                .frame(egui::Frame::default())
                .collapsible(false)
                .show(c.egui, |ui| {
                    let _background_shape = ui.painter().add(egui::Shape::Noop);
                    ui.add_space(18.0);

                    ui.vertical_centered(|ui| {
                        for notification in
                            c.notifications.borrow_mut().notifications.iter()
                        {
                            let font = egui::FontId::new(
                                16.0,
                                egui::FontFamily::Name("james".into()),
                            );

                            ui.add(egui::Label::new(
                                egui::RichText::new(&notification.text)
                                    .font(font)
                                    .color(GOLD.egui()),
                            ));

                            // ui.colored_label(GREEN.egui(), &notification.text);
                        }
                    });
                    ui.add_space(28.0);

                    // let bg = nine_patch_rect_ex(
                    //     egui::Rect::from_min_size(
                    //         ui.clip_rect().left_top(),
                    //         ui.min_size(),
                    //     ),
                    //     c.cached_loader,
                    //     c.egui,
                    //     "panel-horizontal",
                    // );

                    // ui.painter().set(background_shape, bg);
                });
        }
        if cfg!(feature = "dev") &&
            c.config().dev.recording_mode == RecordingMode::None
        {
            let errors = ERRORS.borrow();

            if !errors.data.is_empty() {
                egui::Window::new("Errors")
                    .anchor(egui::Align2::LEFT_TOP, egui::vec2(20.0, 20.0))
                    .show(c.egui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (_, value) in ERRORS.borrow().data.iter() {
                                ui.colored_label(RED.egui(), value.as_ref());
                            }
                        });
                    });
            }
        }

        if cfg!(feature = "dev") &&
            c.config().dev.show_fps &&
            c.config().dev.recording_mode == RecordingMode::None
        {
            let _span = span!("perf counters");

            let dt = c.dt_stats.next(frame_time());
            let fps = c.fps_stats.next(1.0 / frame_time());

            egui::Window::new("Performance")
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(0.0, 0.0))
                .default_width(250.0)
                .show(c.egui, |ui| {
                    let real_fps = get_fps();

                    let fps_color = if real_fps < 55 { RED } else { WHITE };
                    ui.colored_label(
                        fps_color.egui(),
                        format!("FPS (real): {:.0}", real_fps),
                    );

                    ui.separator();

                    ui.label(format!(
                        "Delta mean: {:.0} ms ... std: {:.1} ms",
                        dt.mean * 1000.0,
                        dt.std_dev * 1000.0
                    ));

                    ui.label(format!(
                        "FPS mean: {:.0} ... std: {:.1}",
                        fps.mean, fps.std_dev
                    ));

                    ui.label("percentiles:");
                    ui.label(format!("   50th: {:.0}", fps.percentile_50));
                    ui.label(format!("   75th: {:.0}", fps.percentile_75));
                    ui.label(format!("   90th: {:.0}", fps.percentile_90));
                    ui.label(format!("   95th: {:.0}", fps.percentile_95));
                    ui.label(format!("   99th: {:.0}", fps.percentile_99));

                    ui.separator();
                    if let Some(game_loop) = c.game_loop {
                        game_loop
                            .lock()
                            .performance_metrics(&mut c.world.borrow_mut(), ui);
                    }

                    ui.separator();

                    let mut particles = 0;

                    for (_, particle_system) in
                        c.world_mut().query_mut::<&ParticleSystem>()
                    {
                        particles += particle_system.particles.len();
                    }

                    // TODO: count collision events again?
                    // let mut collision_events = 0;

                    // for (_, collisions) in
                    //     c.world().query::<&Collisions>().iter()
                    // {
                    //     collision_events += collisions.0.len();
                    // }

                    ui.label(format!("Lights: {}", light_count()));
                    ui.label(format!("Particles: {}", particles));
                    ui.label(format!("Entities: {}", c.world().len()));
                    // ui.label(format!("Collision Events: {}", collision_events));

                    ui.separator();

                    ui.label("Perf Counters");

                    ui.separator();

                    for (counter_name, counter) in PerfCounters::global()
                        .counters
                        .iter()
                        .sorted_by(|a, b| a.0.cmp(b.0))
                    {
                        ui.label(format!(
                            "{:<15}: {:<15.0}",
                            counter_name, counter.decayed_average,
                        ));
                    }
                    ui.separator();

                    #[cfg(all(
                        feature = "memory-stats",
                        not(target_arch = "wasm32")
                    ))]
                    {
                        let _span = span!("memory_stats");

                        if let Some(usage) = memory_stats::memory_stats() {
                            ui.label(format!(
                                "Physical Mem: {} MB",
                                usage.physical_mem / (1024 * 1024)
                            ));
                            ui.label(format!(
                                "Virtual Mem: {} MB",
                                usage.virtual_mem / (1024 * 1024)
                            ));
                        } else {
                            ui.label(format!(
                                "Couldn't get the current memory usage :("
                            ));
                        }
                    }

                    #[cfg(feature = "jemalloc")]
                    {
                        let _span = span!("jemalloc stats");
                        ui.separator();

                        ui.label("jemalloc");

                        jemalloc_ctl::epoch::advance().unwrap();

                        let allocated =
                            jemalloc_ctl::stats::allocated::read().unwrap();
                        let resident =
                            jemalloc_ctl::stats::resident::read().unwrap();
                        ui.label(format!(
                            "{} MB allocated",
                            allocated / (1024 * 1024)
                        ));
                        ui.label(format!(
                            "{} MB resident",
                            resident / (1024 * 1024)
                        ));
                    }
                });
        }

        perf_counters_new_frame(c.delta as f64);

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
}
