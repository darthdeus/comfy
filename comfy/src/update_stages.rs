use crate::*;

// TODO: Some of the ordering in the update stages is definitely incorrect.
pub fn run_update_stages(game_loop: &mut dyn GameLoop, c: &mut EngineContext) {
    #[cfg(feature = "exit-after-startup")]
    std::process::exit(0);

    let _span = span!("game-state update");

    #[cfg(any(feature = "quick-exit", feature = "dev"))]
    if is_key_down(KeyCode::F1) && is_key_pressed(KeyCode::Escape) {
        println!("fast exit");
        std::process::exit(0);
    }

    AudioSystem::process_sounds();

    if !*c.is_paused.borrow() {
        set_unpaused_time(get_unpaused_time() + c.delta as f64);
    }

    // TODO: not ideal
    clear_background(BLACK);

    if is_key_pressed(KeyCode::Backquote) &&
        is_key_down(KeyCode::LCtrl) &&
        is_key_down(KeyCode::LAlt)
    {
        let mut config = c.config.borrow_mut();
        config.dev.show_debug = !config.dev.show_debug;
    }

    lighting_parameters_window(c);
    update_child_transforms(c);

    {
        let _span = span!("game_loop.early_update");
        game_loop.early_update(c);
    }

    timings_add_value("delta", delta());

    pause_system(c);
    point_lights_system(c);

    if is_key_pressed(KeyCode::F6) {
        // TODO: bake in some basic sfx into the engine
        play_sound("alarm-next-stage");
        GlobalParams::toggle_flag("debug");
    }

    {
        let _span = span!("game_loop.update");
        game_loop.update(c);
    }

    update_animated_sprites(c);
    update_trails(c);
    update_drawables(c);
    process_sprite_queue(c);
    process_temp_draws(c);
    combat_text_system(c);
    process_notifications(c);
    show_errors(c);
    update_perf_counters(c);

    {
        // TODO: late update maybe should be a bit later?
        let _span = span!("game_loop.late_update");
        game_loop.late_update(c);
    }

    c.draw.borrow_mut().marks.retain_mut(|mark| {
        mark.lifetime -= c.delta;
        mark.lifetime > 0.0
    });

    for mark in c.draw.borrow().marks.iter() {
        draw_circle_z(
            mark.pos.to_world(),
            0.1,
            mark.color,
            90,
            TextureParams {
                blend_mode: BlendMode::Alpha,
                ..Default::default()
            },
        );
    }

    let is_paused =
        *c.is_paused.borrow() || c.flags.borrow_mut().contains(PAUSE_DESPAWN);

    if !is_paused {
        for (entity, despawn) in
            c.world.borrow_mut().query_mut::<&mut DespawnAfter>()
        {
            despawn.0 -= c.delta;

            if despawn.0 <= 0.0 {
                c.to_despawn.borrow_mut().push(entity);
            }
        }
    }

    main_camera_mut().update(c.delta);
    c.commands().run_on(&mut c.world.borrow_mut());
    c.world.borrow_mut().flush();
}

fn update_child_transforms(c: &mut EngineContext) {
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
}

fn update_trails(c: &mut EngineContext) {
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

fn point_lights_system(c: &mut EngineContext) {
    for (_, (transform, light)) in
        c.world_mut().query_mut::<(&Transform, &PointLight)>()
    {
        draw_light(Light::simple(
            transform.position,
            light.radius * light.radius_mod,
            light.strength * light.strength_mod,
        ))
    }
}

fn update_animated_sprites(c: &mut EngineContext) {
    let mut call_queue = vec![];

    if !*c.is_paused.borrow() {
        for (entity, sprite) in c.world().query::<&mut AnimatedSprite>().iter()
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
        call(c);
    }
}

fn pause_system(c: &mut EngineContext) {
    // TODO: configurable pause
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

    if !*c.is_paused.borrow() && !c.flags.borrow().contains(PAUSE_PHYSICS) {
        c.cooldowns.borrow_mut().tick(c.delta);
        c.notifications.borrow_mut().tick(c.delta);
    }
}

fn update_drawables(c: &mut EngineContext) {
    let _span = span!("drawables");

    let mut temp_drawables = vec![];

    std::mem::swap(&mut temp_drawables, &mut c.draw_mut().drawables);

    temp_drawables.retain_mut(|drawable| {
        if let Some(ref mut time) = drawable.time {
            if *time <= 0.0 {
                false
            } else {
                *time -= c.delta;
                (drawable.func)(c);
                true
            }
        } else {
            (drawable.func)(c);
            false
        }
    });

    let mut draw = c.draw_mut();
    std::mem::swap(&mut temp_drawables, &mut draw.drawables);

    for drawable in temp_drawables.drain(..) {
        draw.drawables.push(drawable);
    }
}

fn process_sprite_queue(c: &mut EngineContext) {
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
            draw_sprite_ex(
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

fn process_temp_draws(c: &mut EngineContext) {
    let _span = span!("temp draws");

    for texture in c.draw_mut().textures.drain(..) {
        draw_sprite_ex(
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

fn process_notifications(c: &mut EngineContext) {
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
                            egui::FontFamily::Proportional,
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
}

fn show_errors(c: &mut EngineContext) {
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
}

fn update_perf_counters(c: &mut EngineContext) {
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
}

pub fn lighting_parameters_window(c: &EngineContext) {
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
                            let range = params.floats.get_mut(name).unwrap();
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

                let ints = ["bloom_alg", "physics_substeps", "tonemapping_alg"];

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
                                    .clamp_range(range.min..=(range.max - 1)),
                            );
                        });
                    }
                }
            });
    }
}

pub fn count_to_color(count: i32) -> Color {
    match count {
        0 => WHITE,
        1 => BLUE,
        2 => GREEN,
        3 => RED,
        4 => PURPLE,
        _ => YELLOW,
    }
}
