use winit::event_loop::ControlFlow;

use crate::*;

pub async fn wgpu_game_loop(
    #[cfg(not(target_arch = "wasm32"))] mut loop_helper: LoopHelper,
    mut game_state: Box<dyn RunGameLoop>,
) {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title(game_state.title())
        .with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080))
        // TODO: testing WASM resolution
        // .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(1280, 800));

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

        // web_sys::window().
    }


    let egui_winit = egui_winit::State::new(&event_loop);

    let mut delta = 1.0 / 60.0;
    game_state.set_renderer(WgpuRenderer::new(window, egui_winit).await);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                let _span = span!("frame with vsync");
                #[cfg(not(target_arch = "wasm32"))]
                let _ = loop_helper.loop_start();
                let frame_start = Instant::now();

                {
                    span_with_timing!("frame");
                    game_state.one_frame(delta);
                }

                if game_state.quit_flag() {
                    *control_flow = ControlFlow::Exit;
                }

                set_frame_time(frame_start.elapsed().as_secs_f32());

                let _span = span!("loop_sleep");
                #[cfg(not(target_arch = "wasm32"))]
                loop_helper.loop_sleep();
                delta = frame_start.elapsed().as_secs_f32();
                delta = delta.clamp(1.0 / 300.0, 1.0 / 15.0);

                #[cfg(feature = "tracy")]
                tracy_client::frame_mark();
            }

            Event::WindowEvent { ref event, window_id: _ } => {
                if game_state
                    .renderer()
                    .as_mut_any()
                    .downcast_mut::<WgpuRenderer>()
                    .unwrap()
                    .on_event(event)
                {
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
                        game_state.resize(uvec2(
                            physical_size.width,
                            physical_size.height,
                        ));
                    }

                    WindowEvent::ScaleFactorChanged {
                        new_inner_size, ..
                    } => {
                        game_state.resize(uvec2(
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
