use winit::event_loop::ControlFlow;

use crate::*;

pub async fn run_comfy_main_async(mut game: impl GameLoop + 'static) {
    let _tracy = maybe_setup_tracy();

    #[cfg(not(target_arch = "wasm32"))]
    let target_framerate = game_config().target_framerate;

    #[cfg(not(target_arch = "wasm32"))]
    let mut loop_helper = spin_sleep::LoopHelper::builder()
        .build_with_target_rate(target_framerate);

    let resolution = game_config().resolution;

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
                set_default_shader();

                if game.engine().quit_flag() {
                    *control_flow = ControlFlow::Exit;
                }

                {
                    span_with_timing!("frame");
                    let engine = game.engine();
                    engine.renderer.as_mut().unwrap().begin_frame(egui());

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
                delta = delta.clamp(1.0 / 5000.0, 1.0 / 10.0);

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
                        if physical_size.width > min_resolution.0 &&
                            physical_size.height > min_resolution.1
                        {
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

#[cfg(feature = "use-glow")]
pub use comfy_glow::*;

#[cfg(feature = "use-glow")]
pub async fn glow_game_loop(mut game: impl GameLoop + 'static) {
    let (gl, _shader_version, window, mut event_loop, _context) = {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let gl_attr = video.gl_attr();
        #[cfg(feature = "dev")]
        gl_attr.set_context_flags().debug().set();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let window = video
            .window("FLOAT (GL 3.3)", 1920, 1080)
            .opengl()
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        video.gl_set_swap_interval(0).unwrap();

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                video.gl_get_proc_address(s) as *const _
            })
        };

        let event_loop = sdl.event_pump().unwrap();
        (gl, "#version 330 core", window, event_loop, gl_context)
    };

    let gl = Arc::new(gl);

    let mut delta = 1.0 / 60.0;
    game_state.set_renderer(Box::new(GlowRenderer::new(gl, window)));

    let mut running = true;

    while running {
        let _span = span!("frame");
        #[cfg(not(target_arch = "wasm32"))]
        let _ = loop_helper.loop_start();
        let frame_start = Instant::now();

        for event in event_loop.poll_iter() {
            if !game_state
                .renderer()
                .as_mut_any()
                .downcast_mut::<GlowRenderer>()
                .unwrap()
                .on_event(event.clone())
            {
                match event {
                    sdl2::event::Event::Window { win_event, .. } => match win_event {
                        sdl2::event::WindowEvent::Resized(w, h)
                        | sdl2::event::WindowEvent::SizeChanged(w, h) => {
                            game_state.resize(uvec2(w as u32, h as u32));
                        }
                        // sdl2::event::WindowEvent::Shown => todo!(),
                        // sdl2::event::WindowEvent::Hidden => todo!(),
                        // sdl2::event::WindowEvent::Minimized => todo!(),
                        // sdl2::event::WindowEvent::Maximized => todo!(),
                        // sdl2::event::WindowEvent::Restored => todo!(),
                        // sdl2::event::WindowEvent::Enter => todo!(),
                        // sdl2::event::WindowEvent::Leave => todo!(),
                        // sdl2::event::WindowEvent::FocusGained => todo!(),
                        // sdl2::event::WindowEvent::FocusLost => todo!(),
                        // sdl2::event::WindowEvent::TakeFocus => todo!(),

                        // TODO:
                        // sdl2::event::WindowEvent::Close => todo!(),
                        _ => {}

                    },


                    sdl2::event::Event::Quit { .. } => {
                        // TODO:
                        running = false;
                    }
                    // _ => {
                        // TODO:
                        // engine.process_event(
                        //     window_world_ratio,
                        //     aspect_ratio,
                        //     &mut window,
                        //     event,
                        // );
                    // }

                    sdl2::event::Event::KeyDown {
                        keycode: Some(keycode),
                        repeat: false,
                        ..
                    } => {
                        if let Some(keycode) = KeyCode::try_from_sdl(keycode) {
                            let mut global_state = GLOBAL_STATE.borrow_mut();

                            global_state.pressed.insert(keycode);
                            global_state.just_pressed.insert(keycode);
                        }

                    }
                    sdl2::event::Event::KeyUp {
                        // keycode, keymod, repeat,
                        keycode: Some(keycode),
                        repeat: false,
                        ..
                    } => {
                        if let Some(keycode) = KeyCode::try_from_sdl(keycode) {
                            let mut global_state = GLOBAL_STATE.borrow_mut();

                            global_state.pressed.remove(&keycode);
                            global_state.just_pressed.remove(&keycode);
                        }

                    },
                    sdl2::event::Event::MouseMotion {
                        x, y, xrel, yrel,
                        .. // mousestate,
                    } => {
                        let mut global_state = GLOBAL_STATE.borrow_mut();

                        global_state.mouse_position = vec2(x as f32, y as f32);
                        global_state.mouse_rel = IVec2::new(xrel, yrel);

                        // println!("mouse at {:?}", self.mouse_position);
                    },
                    sdl2::event::Event::MouseButtonDown {
                        mouse_btn,
                        .. // which,
                    } => {
                        if let Some(mouse_btn) = mouse_button_try_from_sdl(mouse_btn) {
                            let mut global_state = GLOBAL_STATE.borrow_mut();

                            global_state.mouse_pressed.insert(mouse_btn);
                            global_state.mouse_just_pressed.insert(mouse_btn);
                        }
                    },
                    sdl2::event::Event::MouseButtonUp {
                        mouse_btn,
                        .. // which,
                    } => {
                        if let Some(mouse_btn) = mouse_button_try_from_sdl(mouse_btn) {
                            let mut global_state = GLOBAL_STATE.borrow_mut();

                            global_state.mouse_pressed.remove(&mouse_btn);
                            global_state.mouse_just_pressed.remove(&mouse_btn);
                            global_state.mouse_just_released.insert(mouse_btn);
                        }

                    }

                    // TODO:
                    sdl2::event::Event::MouseWheel {
                        x,
                        y,
                        ..
                    } => {
                        let mut global_state = GLOBAL_STATE.borrow_mut();

                        global_state.mouse_wheel = (x as f32, y as f32);
                    }

                    _ => {}
                }
            }
        }

        game_state.one_frame(delta);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _span = span!("loop_sleep");
            loop_helper.loop_sleep();
        }

        delta = (Instant::now() - frame_start).as_secs_f32();
    }
}
