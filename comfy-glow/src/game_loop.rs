use crate::prelude::*;

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
