use comfy_core::*;

use sdl2::{
    event::WindowEvent,
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
    mouse::{Cursor, SystemCursor},
};

use egui::{pos2, vec2, CursorIcon, Event, Key, Modifiers};

pub struct EguiIntegration {
    pub ctx: egui::Context,
    pub painter: egui_glow::Painter,
    shapes: Vec<egui::epaint::ClippedShape>,
    textures_delta: egui::TexturesDelta,

    state: EguiStateHandler,
}

impl Drop for EguiIntegration {
    fn drop(&mut self) {
        self.painter.destroy();
    }
}

impl EguiIntegration {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        let ctx = egui::Context::default();
        // ctx.set_pixels_per_point(1.5);

        let painter = egui_glow::Painter::new(
            gl, // Some([resolution.x as i32, resolution.y as i32]),
            "", None,
        )
        .expect("Failed to initialize egui painter");

        Self {
            ctx,
            painter,
            shapes: Default::default(),
            textures_delta: Default::default(),

            state: EguiStateHandler::new(None),
        }
    }

    pub fn begin_frame(&mut self, window: &sdl2::video::Window) {
        self.state.input.pixels_per_point = Some(self.ctx.pixels_per_point());
        let input =
            self.state.take_egui_input(window, self.ctx.pixels_per_point());

        self.ctx.begin_frame(input);
    }

    pub fn end_frame(&mut self, window: &sdl2::video::Window) {
        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            repaint_after: _,
        } = self.ctx.end_frame();

        self.state.process_output(window, &platform_output);

        // TODO: cursor, copy, pixels_per_point

        self.shapes = shapes;
        self.textures_delta.append(textures_delta);
    }

    pub fn process_event(
        &mut self,
        window: &sdl2::video::Window,
        event: sdl2::event::Event,
    ) -> bool {
        // TODO: pixels per point
        input_to_egui(window, event, &self.ctx, &mut self.state)
    }

    pub fn paint(&mut self, dimensions: UVec2) {
        let shapes = std::mem::take(&mut self.shapes);
        let mut textures_delta = std::mem::take(&mut self.textures_delta);

        for (id, image_delta) in textures_delta.set {
            self.painter.set_texture(id, &image_delta);
        }
        let clipped_primitives = self.ctx.tessellate(shapes);

        self.painter.paint_primitives(
            [dimensions.x, dimensions.y],
            self.ctx.pixels_per_point(),
            clipped_primitives.as_slice(),
        );

        for id in textures_delta.free.drain(..) {
            self.painter.free_texture(id);
        }
    }
}

pub struct FusedCursor {
    pub cursor: Cursor,
    pub icon: SystemCursor,
}

impl FusedCursor {
    pub fn new() -> Self {
        Self {
            cursor: Cursor::from_system(SystemCursor::Arrow).unwrap(),
            icon: SystemCursor::Arrow,
        }
    }
}

impl Default for FusedCursor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EguiStateHandler {
    pub fused_cursor: FusedCursor,
    pub pointer_pos: egui::Pos2,
    pub input: egui::RawInput,
    pub modifiers: Modifiers,
    pub start_time: Instant,
}

impl EguiStateHandler {
    pub fn new(screen_rect: Option<egui::Rect>) -> EguiStateHandler {
        Self {
            fused_cursor: FusedCursor::default(),
            pointer_pos: egui::Pos2::new(0f32, 0f32),
            input: egui::RawInput { screen_rect, ..Default::default() },
            modifiers: Modifiers::default(),
            start_time: Instant::now(),
        }
    }

    pub fn process_output(
        &mut self,
        window: &sdl2::video::Window,
        egui_output: &egui::PlatformOutput,
    ) {
        if !egui_output.copied_text.is_empty() {
            let copied_text = egui_output.copied_text.clone();
            {
                let result = window
                    .subsystem()
                    .clipboard()
                    .set_clipboard_text(&copied_text);
                if result.is_err() {
                    dbg!("Unable to set clipboard content to SDL clipboard.");
                }
            }
        }

        translate_cursor(&mut self.fused_cursor, egui_output.cursor_icon);
    }

    /// Prepare for a new frame by extracting the accumulated input,
    /// as well as setting [the time](egui::RawInput::time) and [screen rectangle](egui::RawInput::screen_rect).
    pub fn take_egui_input(
        &mut self,
        window: &sdl2::video::Window,
        pixels_per_point: f32,
    ) -> egui::RawInput {
        // TODO: pixels per point properly
        // let pixels_per_point = self.pixels_per_point();
        // let pixels_per_point = 1.0;

        self.input.time = Some(self.start_time.elapsed().as_secs_f64());

        // On Windows, a minimized window will have 0 width and height.
        // See: https://github.com/rust-windowing/winit/issues/208
        // This solves an issue where egui window positions would be changed when minimizing on Windows.
        let dims = window.drawable_size();

        let screen_size_in_pixels = egui::vec2(dims.0 as f32, dims.1 as f32); //screen_size_in_pixels(window);
        let screen_size_in_points = screen_size_in_pixels / pixels_per_point;
        self.input.screen_rect =
            if screen_size_in_points.x > 0.0 && screen_size_in_points.y > 0.0 {
                Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    screen_size_in_points,
                ))
            } else {
                None
            };

        self.input.take()
    }
}

pub fn input_to_egui(
    window: &sdl2::video::Window,
    event: sdl2::event::Event,
    ctx: &egui::Context,
    state: &mut EguiStateHandler,
) -> bool {
    use sdl2::event::Event::*;

    if event.get_window_id() != Some(window.id()) {
        return false;
    }
    match event {
        // handle when window Resized and SizeChanged.
        Window { win_event, .. } => {
            match win_event {
                WindowEvent::Resized(_, _) |
                sdl2::event::WindowEvent::SizeChanged(_, _) => {
                    // println!("TODO: resizing not implemented");
                    false
                    // painter.update_screen_rect(window.drawable_size());
                    // state.input.screen_rect = Some(painter.screen_rect);
                }
                _ => false,
            }
        }

        //MouseButonLeft pressed is the only one needed by egui
        MouseButtonDown { mouse_btn, .. } => {
            let mouse_btn = match mouse_btn {
                MouseButton::Left => Some(egui::PointerButton::Primary),
                MouseButton::Middle => Some(egui::PointerButton::Middle),
                MouseButton::Right => Some(egui::PointerButton::Secondary),
                _ => None,
            };
            if let Some(pressed) = mouse_btn {
                state.input.events.push(egui::Event::PointerButton {
                    pos: state.pointer_pos,
                    button: pressed,
                    pressed: true,
                    modifiers: state.modifiers,
                });
            }

            ctx.wants_pointer_input()
        }

        //MouseButonLeft pressed is the only one needed by egui
        MouseButtonUp { mouse_btn, .. } => {
            let mouse_btn = match mouse_btn {
                MouseButton::Left => Some(egui::PointerButton::Primary),
                MouseButton::Middle => Some(egui::PointerButton::Middle),
                MouseButton::Right => Some(egui::PointerButton::Secondary),
                _ => None,
            };
            if let Some(released) = mouse_btn {
                state.input.events.push(egui::Event::PointerButton {
                    pos: state.pointer_pos,
                    button: released,
                    pressed: false,
                    modifiers: state.modifiers,
                });
            }

            ctx.wants_pointer_input()
        }

        MouseMotion { x, y, .. } => {
            state.pointer_pos = pos2(
                x as f32 / ctx.pixels_per_point(),
                y as f32 / ctx.pixels_per_point(),
            );
            state
                .input
                .events
                .push(egui::Event::PointerMoved(state.pointer_pos));

            // ctx.wants_pointer_input()
            false
        }

        KeyUp { keycode, keymod, .. } => {
            let key_code = match keycode {
                Some(key_code) => key_code,
                _ => return false,
            };
            let key = match translate_virtual_key_code(key_code) {
                Some(key) => key,
                _ => return false,
            };
            state.modifiers = Modifiers {
                alt: (keymod & Mod::LALTMOD == Mod::LALTMOD) ||
                    (keymod & Mod::RALTMOD == Mod::RALTMOD),
                ctrl: (keymod & Mod::LCTRLMOD == Mod::LCTRLMOD) ||
                    (keymod & Mod::RCTRLMOD == Mod::RCTRLMOD),
                shift: (keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD) ||
                    (keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD),
                mac_cmd: keymod & Mod::LGUIMOD == Mod::LGUIMOD,

                //TOD: Test on both windows and mac
                command: (keymod & Mod::LCTRLMOD == Mod::LCTRLMOD) ||
                    (keymod & Mod::LGUIMOD == Mod::LGUIMOD),
            };

            state.input.events.push(egui::Event::Key {
                key,
                pressed: false,
                modifiers: state.modifiers,
                repeat: false, // TODO
            });

            false
        }

        KeyDown { keycode, keymod, .. } => {
            let key_code = match keycode {
                Some(key_code) => key_code,
                _ => return false,
            };

            let key = match translate_virtual_key_code(key_code) {
                Some(key) => key,
                _ => return false,
            };
            state.modifiers = Modifiers {
                alt: (keymod & Mod::LALTMOD == Mod::LALTMOD) ||
                    (keymod & Mod::RALTMOD == Mod::RALTMOD),
                ctrl: (keymod & Mod::LCTRLMOD == Mod::LCTRLMOD) ||
                    (keymod & Mod::RCTRLMOD == Mod::RCTRLMOD),
                shift: (keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD) ||
                    (keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD),
                mac_cmd: keymod & Mod::LGUIMOD == Mod::LGUIMOD,

                //TOD: Test on both windows and mac
                command: (keymod & Mod::LCTRLMOD == Mod::LCTRLMOD) ||
                    (keymod & Mod::LGUIMOD == Mod::LGUIMOD),
            };

            state.input.events.push(egui::Event::Key {
                key,
                pressed: true,
                modifiers: state.modifiers,
                repeat: false, // TODO
            });

            if state.modifiers.command && key == Key::C {
                // println!("copy event");
                state.input.events.push(Event::Copy);
            } else if state.modifiers.command && key == Key::X {
                // println!("cut event");
                state.input.events.push(Event::Cut);
            } else if state.modifiers.command && key == Key::V {
                // println!("paste");
                if let Ok(contents) =
                    window.subsystem().clipboard().clipboard_text()
                {
                    state.input.events.push(Event::Text(contents));
                }
            }
            false
        }

        TextInput { text, .. } => {
            state.input.events.push(Event::Text(text));
            false
        }

        MouseWheel { x, y, .. } => {
            let delta = vec2(x as f32 * 8.0, y as f32 * 8.0);
            let sdl = window.subsystem().sdl();
            if sdl.keyboard().mod_state() & Mod::LCTRLMOD == Mod::LCTRLMOD ||
                sdl.keyboard().mod_state() & Mod::RCTRLMOD == Mod::RCTRLMOD
            {
                state.input.events.push(Event::Zoom((delta.y / 125.0).exp()));
            } else {
                state.input.events.push(Event::Scroll(delta));
            }
            false
        }

        _ => {
            false
            //dbg!(event);
        }
    }
}

pub fn translate_virtual_key_code(
    key: sdl2::keyboard::Keycode,
) -> Option<egui::Key> {
    use Keycode::*;

    Some(match key {
        Left => Key::ArrowLeft,
        Up => Key::ArrowUp,
        Right => Key::ArrowRight,
        Down => Key::ArrowDown,

        Escape => Key::Escape,
        Tab => Key::Tab,
        Backspace => Key::Backspace,
        Space => Key::Space,
        Return => Key::Enter,

        Insert => Key::Insert,
        Home => Key::Home,
        Delete => Key::Delete,
        End => Key::End,
        PageDown => Key::PageDown,
        PageUp => Key::PageUp,

        Kp0 | Num0 => Key::Num0,
        Kp1 | Num1 => Key::Num1,
        Kp2 | Num2 => Key::Num2,
        Kp3 | Num3 => Key::Num3,
        Kp4 | Num4 => Key::Num4,
        Kp5 | Num5 => Key::Num5,
        Kp6 | Num6 => Key::Num6,
        Kp7 | Num7 => Key::Num7,
        Kp8 | Num8 => Key::Num8,
        Kp9 | Num9 => Key::Num9,

        A => Key::A,
        B => Key::B,
        C => Key::C,
        D => Key::D,
        E => Key::E,
        F => Key::F,
        G => Key::G,
        H => Key::H,
        I => Key::I,
        J => Key::J,
        K => Key::K,
        L => Key::L,
        M => Key::M,
        N => Key::N,
        O => Key::O,
        P => Key::P,
        Q => Key::Q,
        R => Key::R,
        S => Key::S,
        T => Key::T,
        U => Key::U,
        V => Key::V,
        W => Key::W,
        X => Key::X,
        Y => Key::Y,
        Z => Key::Z,

        _ => {
            return None;
        }
    })
}

pub fn translate_cursor(
    fused: &mut FusedCursor,
    cursor_icon: egui::CursorIcon,
) {
    let tmp_icon = match cursor_icon {
        CursorIcon::Crosshair => SystemCursor::Crosshair,
        CursorIcon::Default => SystemCursor::Arrow,
        CursorIcon::Grab => SystemCursor::Hand,
        CursorIcon::Grabbing => SystemCursor::SizeAll,
        CursorIcon::Move => SystemCursor::SizeAll,
        CursorIcon::PointingHand => SystemCursor::Hand,
        CursorIcon::ResizeHorizontal => SystemCursor::SizeWE,
        CursorIcon::ResizeNeSw => SystemCursor::SizeNESW,
        CursorIcon::ResizeNwSe => SystemCursor::SizeNWSE,
        CursorIcon::ResizeVertical => SystemCursor::SizeNS,
        CursorIcon::Text => SystemCursor::IBeam,
        CursorIcon::NotAllowed | CursorIcon::NoDrop => SystemCursor::No,
        CursorIcon::Wait => SystemCursor::Wait,
        //There doesn't seem to be a suitable SDL equivalent...
        _ => SystemCursor::Arrow,
    };

    if tmp_icon != fused.icon {
        fused.cursor = Cursor::from_system(tmp_icon).unwrap();
        fused.icon = tmp_icon;
        fused.cursor.set();
    }
}
