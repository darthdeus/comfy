use crate::*;

#[derive(Clone, Debug)]
pub struct SimpleAnimation {
    pub name: Cow<'static, str>,
    pub sheet: Spritesheet,
    pub current_frame: usize,
    pub elapsed_time: f32,
    pub frame_time: f32,
    pub frame_range: Option<(usize, usize)>,
}

impl SimpleAnimation {
    // pub fn new(name: impl Into<Cow<'static, str>>, sheet: Spritesheet) -> Self {
    //     let frames = sheet.rows * sheet.columns;
    //     let animation_time = 10.0;
    //     let frame_time = animation_time / frames as f32;
    //
    //     Self {
    //         name: name.into(),
    //         sheet,
    //         current_frame: gen_range(0, frames),
    //         elapsed_time: 0.0,
    //         frame_time,
    //     }
    // }

    pub fn new(
        name: impl Into<Cow<'static, str>>,
        sheet: Spritesheet,
        frame_range: Option<(usize, usize)>,
    ) -> Self {
        let frames = frame_range
            .map_or(sheet.rows * sheet.columns, |(start, end)| end - start + 1);
        let animation_time = 0.5;
        let frame_time = animation_time / frames as f32;

        Self {
            name: name.into(),
            sheet,
            current_frame: frame_range.map_or(
                gen_range(0, sheet.rows * sheet.columns),
                |(start, end)| gen_range(start, end + 1),
            ),
            elapsed_time: 0.0,
            frame_time,
            frame_range,
        }
    }

    // pub fn update(&mut self, delta: f32) {
    //     self.elapsed_time += delta;
    //
    //     if self.elapsed_time >= self.frame_time {
    //         self.current_frame = (self.current_frame + 1) %
    //             (self.sheet.rows * self.sheet.columns);
    //         self.elapsed_time = 0.0;
    //     }
    // }

    pub fn update(&mut self, delta: f32) {
        self.elapsed_time += delta;

        if self.elapsed_time >= self.frame_time {
            let total_frames = self.sheet.rows * self.sheet.columns;
            let (start_frame, end_frame) =
                self.frame_range.unwrap_or((0, total_frames - 1));

            self.current_frame = start_frame +
                ((self.current_frame - start_frame + 1) %
                    (end_frame - start_frame + 1));
            self.elapsed_time -= self.frame_time;
        }
    }

    pub fn draw(&self, position: Vec2, z_index: i32, size: f32, rotation: f32) {
        let texture = texture_id(&self.name);

        let image_size = match Assets::image_size(texture) {
            ImageSizeResult::Loaded(size) => size,
            ImageSizeResult::LoadingInProgress => {
                return;
            }
            ImageSizeResult::ImageNotFound => {
                error!("NO SIZE FOR TEXTURE {:?}", texture);
                UVec2::ONE
            }
        }
        .as_ivec2();

        // TODO: ratio should be a part of draw_sprite_ex/pro
        let ratio = (image_size.x / self.sheet.columns as i32) as f32 /
            (image_size.y / self.sheet.rows as i32) as f32;

        draw_sprite_ex(texture, position, WHITE, z_index, DrawTextureParams {
            dest_size: Some(vec2(size, size / ratio).as_world_size()),
            source_rect: Some(self.current_frame(image_size)),
            rotation,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }

    pub fn current_frame(&self, sprite_size: IVec2) -> IRect {
        // let frame_width = self.sheet.width / self.sheet.columns;
        // let frame_height = self.sheet.height / self.sheet.rows;

        let frame_width = sprite_size.x as usize / self.sheet.columns;
        let frame_height = sprite_size.y as usize / self.sheet.rows;

        // let frame_width = 256;
        // let frame_height = 256;

        let row = self.current_frame / self.sheet.columns;
        let column = self.current_frame % self.sheet.columns;

        let offset_x = column * frame_width;
        let offset_y = row * frame_height;

        IRect::new(
            ivec2(offset_x as i32, offset_y as i32),
            ivec2(frame_width as i32, frame_height as i32),
        )
    }
}
