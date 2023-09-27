use winit::event_loop::ControlFlow;

use crate::*;

pub async fn run_comfy_main_async(mut game: impl GameLoop + 'static) {
    let _tracy = maybe_setup_tracy();

    #[cfg(not(target_arch = "wasm32"))]
    let target_framerate = 60;

    #[cfg(not(target_arch = "wasm32"))]
    let mut loop_helper = spin_sleep::LoopHelper::builder()
        .build_with_target_rate(target_framerate);

    let resolution = game.engine().config.get_mut().resolution;

    let event_loop = winit::event_loop::EventLoop::new();
    let window =
        winit::window::WindowBuilder::new().with_title(game.engine().title());

    let window = match resolution {
        ResolutionConfig::Physical(w, h) => {
            window.with_inner_size(winit::dpi::PhysicalSize::new(w, h))
        }

        ResolutionConfig::Logical(w, h) => {
            window.with_inner_size(winit::dpi::LogicalSize::new(w, h))
        }
    };


    let window = window.build(&event_loop).unwrap();

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
                let dst = doc.get_element_by_id("wasm-body")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    info!("scale factor = {}", window.scale_factor());

    let egui_winit = egui_winit::State::new(&event_loop);

    let mut delta = 1.0 / 60.0;

    let renderer = WgpuRenderer::new(window, egui_winit).await;
    game.engine().texture_creator = Some(renderer.texture_creator.clone());
    game.engine().renderer = Some(renderer);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                let _span = span!("frame with vsync");
                #[cfg(not(target_arch = "wasm32"))]
                let _ = loop_helper.loop_start();
                let frame_start = Instant::now();

                set_delta(delta);
                set_time(get_time() + delta as f64);

                if game.engine().quit_flag() {
                    *control_flow = ControlFlow::Exit;
                }

                {
                    span_with_timing!("frame");
                    let engine = game.engine();
                    engine.renderer.as_mut().unwrap().begin_frame(&engine.egui);

                    game.engine().frame += 1;
                    game.update();
                }

                {
                    let mut global_state = GLOBAL_STATE.borrow_mut();
                    global_state.just_pressed.clear();
                    global_state.just_released.clear();
                    global_state.mouse_just_pressed.clear();
                    global_state.mouse_just_released.clear();
                    global_state.mouse_wheel = (0.0, 0.0);
                }

                set_frame_time(frame_start.elapsed().as_secs_f32());
                inc_frame_num();

                let _span = span!("loop_sleep");
                #[cfg(not(target_arch = "wasm32"))]
                loop_helper.loop_sleep();
                delta = frame_start.elapsed().as_secs_f32();
                delta = delta.clamp(1.0 / 300.0, 1.0 / 15.0);

                #[cfg(feature = "tracy")]
                tracy_client::frame_mark();
            }

            Event::WindowEvent { ref event, window_id: _ } => {
                if game.engine().on_event(event) {
                    return;
                }

                match event {
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput { state, virtual_keycode, .. },
                        ..
                    } => {
                        if let Some(keycode) =
                            virtual_keycode.and_then(KeyCode::try_from_winit)
                        {
                            match state {
                                ElementState::Pressed => {
                                    let mut state = GLOBAL_STATE.borrow_mut();

                                    state.pressed.insert(keycode);
                                    state.just_pressed.insert(keycode);
                                    state.just_released.remove(&keycode);
                                }

                                ElementState::Released => {
                                    let mut state = GLOBAL_STATE.borrow_mut();

                                    state.pressed.remove(&keycode);
                                    state.just_pressed.remove(&keycode);
                                    state.just_released.insert(keycode);
                                }
                            }
                        }
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        GLOBAL_STATE.borrow_mut().mouse_position =
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
                        };

                        let mut global_state = GLOBAL_STATE.borrow_mut();

                        match state {
                            ElementState::Pressed => {
                                global_state.mouse_pressed.insert(quad_button);
                                global_state
                                    .mouse_just_pressed
                                    .insert(quad_button);
                            }
                            ElementState::Released => {
                                global_state.mouse_pressed.remove(&quad_button);
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

                    WindowEvent::Resized(physical_size) => {
                        if physical_size.width > 0  && physical_size.height > 0 {
                            game.engine().resize(uvec2(
                                physical_size.width,
                                physical_size.height,
                            ));
                        }
                    }

                    WindowEvent::ScaleFactorChanged {
                        new_inner_size, ..
                    } => {
                        game.engine().resize(uvec2(
                            new_inner_size.width,
                            new_inner_size.height,
                        ));
                    }

                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}
