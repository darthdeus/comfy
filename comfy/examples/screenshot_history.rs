use comfy::*;

comfy_game!("Screenshot History Example", ScreenshotHistoryExample);

pub struct ScreenshotHistoryExample {
    pub initialized: bool,
    pub handles: Vec<TextureHandle>,
}

impl GameLoop for ScreenshotHistoryExample {
    fn new(_c: &mut EngineState) -> Self {
        Self { initialized: false, handles: Vec::new() }
    }

    fn update(&mut self, c: &mut EngineContext) {
        let screen_size = uvec2(screen_width() as u32, screen_height() as u32);

        if !self.initialized {
            self.initialized = true;

            c.renderer.screenshot_params.record_screenshots = true;
            c.renderer.screenshot_params.screenshot_interval_n = 10;
            c.renderer.screenshot_params.history_length = 5;


            for i in 0..c.renderer.screenshot_params.history_length {
                self.handles.push(
                    c.renderer
                        .context
                        .texture_creator
                        .borrow_mut()
                        .handle_from_size(
                            &format!("screenshot-{i}"),
                            screen_size,
                            RED,
                        ),
                );
            }
        }

        for (screenshot, handle) in
            c.renderer.screenshot_history_buffer.iter().zip(self.handles.iter())
        {
            c.renderer
                .context
                .texture_creator
                .borrow_mut()
                .update_texture(screenshot, *handle);
        }

        // draw history
        for (i, handle) in self.handles.iter().enumerate() {
            draw_sprite(
                *handle,
                vec2(i as f32 * 2.0 + 2.0, 2.0),
                WHITE,
                100,
                splat(2.0),
            );
        }

        let time = get_time() as f32;

        clear_background(Color::rgb8(13, 2, 8));

        let colors = [RED, GREEN, BLUE, YELLOW, CYAN];

        for (i, color) in colors.into_iter().enumerate() {
            let s = (i + 1) as f32;
            let t = s * time;

            let z_index = i as i32;

            draw_circle(
                vec2(i as f32 * 2.0 + 2.0, t.sin() - 2.0),
                s * 0.75,
                color,
                z_index,
            );

            let r = rescale(i, 0..colors.len(), 2..5);
            draw_arc(
                vec2(-s - 2.0, t.sin() - 2.0),
                r,
                PI - t.sin(),
                PI - t.cos(),
                color,
                z_index,
            );

            draw_arc_outline(
                vec2(-0.5, s - 0.5),
                r,
                s / 10.0,
                PI / 4.0 - t.cos().powf(2.0),
                t.sin().powf(2.0) + PI * 3.0 / 4.0,
                // t.cos(),
                color,
                z_index,
            );
        }
    }
}
