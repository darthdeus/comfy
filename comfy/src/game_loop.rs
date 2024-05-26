use comfy_core::winit::event::DeviceEvent;
use winit::event_loop::ControlFlow;

use crate::*;

pub async fn run_comfy_main_async(
    mut game: impl GameLoop + 'static,
    mut engine: EngineState,
) {
    let _tracy = maybe_setup_tracy();

    #[cfg(not(target_arch = "wasm32"))]
    let target_framerate = game_config().target_framerate;

    #[cfg(not(target_arch = "wasm32"))]
    let mut loop_helper = spin_sleep::LoopHelper::builder()
        .build_with_target_rate(target_framerate);

    let resolution = {
        use std::env::var;

        match (
            var("COMFY_RES_WIDTH").map(|x| x.parse::<u32>()),
            var("COMFY_RES_HEIGHT").map(|x| x.parse::<u32>()),
        ) {
            (Ok(Ok(width)), Ok(Ok(height))) => {
                ResolutionConfig::Physical(width, height)
            }
            _ => game_config().resolution,
        }
    };

    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let title = {
        let game_name = game_config().game_name.clone();
        let dev_name = format!("{} (Comfy Engine DEV BUILD)", game_name);

        match std::env::var("COMFY_DEV_TITLE") {
            Ok(_) => dev_name,
            Err(_) => {
                cfg_if! {
                    if #[cfg(feature = "dev")] {
                        dev_name
                    } else {
                        game_name
                    }
                }
            }
        }
    };

    let window = winit::window::WindowBuilder::new().with_title(title);

    let mut window = match resolution {
        ResolutionConfig::Physical(w, h) => {
            window.with_inner_size(winit::dpi::PhysicalSize::new(w, h))
        }

        ResolutionConfig::Logical(w, h) => {
            window.with_inner_size(winit::dpi::LogicalSize::new(w, h))
        }
    };

    if game_config().fullscreen {
        window = window
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
    }

    let window = window.build(&event_loop).unwrap();
    let window = Box::leak(Box::new(window));

    let min_resolution = match game_config_mut()
        .min_resolution
        .ensure_non_zero()
    {
        ResolutionConfig::Physical(w, h) => {
            window
                .set_min_inner_size(Some(winit::dpi::PhysicalSize::new(w, h)));
            (w, h)
        }
        ResolutionConfig::Logical(w, h) => {
            window.set_min_inner_size(Some(winit::dpi::LogicalSize::new(w, h)));
            (w, h)
        }
    };

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(
            resolution.width(),
            resolution.height(),
        ));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                match &game_config().wasm_append_id {
                    Some(id) => {
                        let dst = doc.get_element_by_id(&id)?;
                        let canvas = web_sys::Element::from(window.canvas());
                        dst.append_child(&canvas).ok()?;
                    }
                    _ => {
                        let dst = doc.body()?;
                        let canvas = web_sys::Element::from(window.canvas());
                        dst.append_child(&canvas).ok()?;
                    }
                };

                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }


    let scale_factor = game_config()
        .scale_factor_override
        .unwrap_or(window.scale_factor() as f32);

    let egui_winit = egui_winit::State::new(
        // TODO: this is wrong, since we'll likely just end up with two contexts,
        // but for now trying to just get things to compile
        egui().clone(),
        egui().viewport_id(),
        &window,
        Some(scale_factor as f32),
        None,
    );

    let mut delta = 1.0 / 60.0;

    let renderer = WgpuRenderer::new(window, egui_winit).await;

    engine.texture_creator = Some(renderer.texture_creator.clone());
    engine.renderer = Some(renderer);

    event_loop
        .run(move |event, control_flow| {
            match event {
                Event::AboutToWait => {
                    let _span = span!("frame with vsync");
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = loop_helper.loop_start();
                    let frame_start = Instant::now();

                    set_delta(delta);
                    set_time(get_time() + delta as f64);
                    use_default_shader();

                    if engine.quit_flag {
                        control_flow.exit();
                    }

                    {
                        span_with_timing!("frame");
                        {
                            let _span = span!("begin_frame");
                            let renderer = engine.renderer.as_mut().unwrap();

                            egui().begin_frame(
                                renderer
                                    .egui_winit
                                    .take_egui_input(&renderer.window),
                            );
                        }

                        engine.frame += 1;

                        // All internal engine code expect an `EngineContext`.
                        let mut c = engine.make_context();
                        run_early_update_stages(&mut c);
                        game.update(&mut c);
                        update_perf_counters(&mut c, &game);
                        run_late_update_stages(&mut c, delta);
                    }

                    {
                        let mut global_state = GLOBAL_STATE.borrow_mut();
                        global_state.just_pressed.clear();
                        global_state.just_released.clear();
                        global_state.mouse_just_pressed.clear();
                        global_state.mouse_just_released.clear();
                        global_state.mouse_wheel = (0.0, 0.0);
                        global_state.mouse_input_this_frame = false;
                        global_state.mouse_moved_this_frame = false;

                        engine
                            .renderer
                            .as_ref()
                            .unwrap()
                            .window
                            .set_cursor_visible(!global_state.cursor_hidden);
                    }


                    set_frame_time(frame_start.elapsed().as_secs_f32());
                    inc_frame_num();

                    let _span = span!("loop_sleep");
                    #[cfg(not(target_arch = "wasm32"))]
                    loop_helper.loop_sleep();
                    delta = frame_start.elapsed().as_secs_f32();
                    delta = delta.clamp(1.0 / 5000.0, 1.0 / 10.0);
                    #[cfg(not(target_arch = "wasm32"))]
                    loop_helper.set_target_rate(game_config().target_framerate);

                    #[cfg(feature = "tracy")]
                    tracy_client::frame_mark();
                }

                Event::WindowEvent { ref event, window_id: _ } => {
                    if engine.renderer.as_mut().unwrap().on_event(event, egui())
                    {
                        return;
                    }

                    match event {
                        WindowEvent::CursorMoved { position, .. } => {
                            let mut global_state = GLOBAL_STATE.borrow_mut();

                            global_state.mouse_input_this_frame = true;
                            global_state.mouse_moved_this_frame = true;
                            global_state.mouse_position =
                                vec2(position.x as f32, position.y as f32);
                        }

                        WindowEvent::MouseInput { state, button, .. } => {
                            let quad_button = match button {
                                winit::event::MouseButton::Left => {
                                    MouseButton::Left
                                }
                                winit::event::MouseButton::Right => {
                                    MouseButton::Right
                                }
                                winit::event::MouseButton::Middle => {
                                    MouseButton::Middle
                                }
                                winit::event::MouseButton::Other(num) => {
                                    MouseButton::Other(*num)
                                }
                                winit::event::MouseButton::Back => {
                                    MouseButton::Back
                                }
                                winit::event::MouseButton::Forward => {
                                    MouseButton::Forward
                                }
                            };

                            let mut global_state = GLOBAL_STATE.borrow_mut();

                            global_state.mouse_input_this_frame = true;

                            match state {
                                ElementState::Pressed => {
                                    global_state
                                        .mouse_pressed
                                        .insert(quad_button);
                                    global_state
                                        .mouse_just_pressed
                                        .insert(quad_button);
                                }
                                ElementState::Released => {
                                    global_state
                                        .mouse_pressed
                                        .remove(&quad_button);
                                    global_state
                                        .mouse_just_pressed
                                        .remove(&quad_button);
                                    global_state
                                        .mouse_just_released
                                        .insert(quad_button);
                                }
                            }
                        }

                        WindowEvent::MouseWheel { delta, .. } => {
                            let mut global_state = GLOBAL_STATE.borrow_mut();

                            global_state.mouse_input_this_frame = true;

                            match delta {
                                MouseScrollDelta::LineDelta(x, y) => {
                                    global_state.mouse_wheel = (*x, *y);
                                }
                                MouseScrollDelta::PixelDelta(delta) => {
                                    error!(
                                        "MouseScrollDelta::PixelDelta not \
                                         implemented! {:?}",
                                        delta
                                    );
                                }
                            }
                        }
                        
                        WindowEvent::KeyboardInput {event, ..} => {
                            if let Some(keycode) =
                                KeyCode::try_from_winit(event.physical_key)
                            {
                                match event.state {
                                    ElementState::Pressed => {
                                        let mut state =
                                            GLOBAL_STATE.borrow_mut();

                                        state.pressed.insert(keycode);
                                        state.just_pressed.insert(keycode);
                                        state.just_released.remove(&keycode);
                                    }

                                    ElementState::Released => {
                                        let mut state =
                                            GLOBAL_STATE.borrow_mut();

                                        state.pressed.remove(&keycode);
                                        state.just_pressed.remove(&keycode);
                                        state.just_released.insert(keycode);
                                    }
                                }
                            }
                        }

                        WindowEvent::Resized(physical_size) => {
                            if physical_size.width > min_resolution.0 &&
                                physical_size.height > min_resolution.1
                            {
                                engine.resize(uvec2(
                                    physical_size.width,
                                    physical_size.height,
                                ));
                            }
                        }

                        // TODO: handle new scale factor
                        // WindowEvent::ScaleFactorChanged {
                        //     new_inner_size, ..
                        // } => {
                        //     engine.resize(uvec2(
                        //         new_inner_size.width,
                        //         new_inner_size.height,
                        //     ));
                        // }
                        WindowEvent::CloseRequested => {
                            control_flow.exit();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
