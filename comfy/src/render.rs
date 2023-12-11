use crate::*;

#[derive(Clone, Debug)]
pub struct Sprite {
    pub name: Cow<'static, str>,
    pub size: Vec2,
    pub z_index: i32,
    pub color: Color,
    pub blend_mode: BlendMode,
    pub source_rect: Option<IRect>,

    pub offset: Vec2,
    pub rotation_x: f32,

    pub flip_x: bool,
    pub flip_y: bool,
}

impl Sprite {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        size: Vec2,
        z_index: i32,
        color: Color,
    ) -> Self {
        Self {
            name: name.into(),
            size,
            z_index,
            color,
            blend_mode: BlendMode::None,
            source_rect: None,
            offset: Vec2::ZERO,
            rotation_x: 0.0,
            flip_x: false,
            flip_y: false,
        }
    }

    pub fn with_blend_mode(self, blend_mode: BlendMode) -> Self {
        Self { blend_mode, ..self }
    }

    pub fn with_rect(self, x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { source_rect: Some(IRect::new(ivec2(x, y), ivec2(w, h))), ..self }
    }

    pub fn with_rotation_x(self, rotation_x: f32) -> Self {
        Self { rotation_x, ..self }
    }

    pub fn with_z_index(self, z_index: i32) -> Self {
        Self { z_index, ..self }
    }

    pub fn set_rect(self, source_rect: Option<IRect>) -> Self {
        Self { source_rect, ..self }
    }

    pub fn to_quad_draw(&self, transform: &Transform) -> QuadDraw {
        QuadDraw {
            texture: texture_id(&self.name),
            transform: *transform,
            z_index: self.z_index,
            color: self.color,
            blend_mode: self.blend_mode,
            dest_size: self.size * transform.scale,
            source_rect: self.source_rect,
            rotation_x: self.rotation_x,
            flip_x: self.flip_x,
            flip_y: self.flip_y,
        }
    }
}


pub fn fhd_ratio() -> f32 {
    1920.0 / 1080.0
}

#[allow(dead_code)]
pub fn fhd_resize_ratio() -> f32 {
    let ratio_w = screen_width() / 1920.0;
    let ratio_h = screen_height() / 1080.0;

    ratio_w.min(ratio_h)
}

pub struct QueuedTexture {
    pub texture: TextureHandle,
    pub position: Position,
    pub z_index: i32,
    pub color: Color,
    pub params: DrawTextureParams,
}

pub struct QueuedLine {
    pub start: Position,
    pub end: Position,
    pub width: f32,
    pub z_index: i32,
    pub color: Color,
}

pub struct Drawable {
    pub func: Box<dyn Fn(&mut EngineContext) + Send + Sync + 'static>,
    pub time: Option<f32>,
}

static GLOBAL_DRAW: Lazy<AtomicRefCell<Draw>> =
    Lazy::new(|| AtomicRefCell::new(Draw::new()));

pub fn draw_mut() -> AtomicRefMut<'static, Draw> {
    GLOBAL_DRAW.borrow_mut()
}

pub struct Draw {
    // pub props: TextureHandle,
    pub marks: Vec<DebugMark>,
    pub circles: Vec<(Position, f32, Color)>,
    pub textures: Vec<QueuedTexture>,
    pub lines: Vec<QueuedLine>,
    pub texts: Vec<(String, Vec2, Color, f32)>,

    pub drawables: Vec<Drawable>,
}

impl Draw {
    pub fn new() -> Self {
        Self {
            // props,
            marks: vec![],
            circles: vec![],
            textures: vec![],
            lines: vec![],
            texts: vec![],
            drawables: vec![],
        }
    }

    pub fn once(
        &mut self,
        func: impl Fn(&mut EngineContext) + 'static + Send + Sync,
    ) {
        self.drawables.push(Drawable { func: Box::new(func), time: None });
    }

    pub fn timed(
        &mut self,
        time: f32,
        func: impl Fn(&mut EngineContext) + 'static + Send + Sync,
    ) {
        self.drawables
            .push(Drawable { func: Box::new(func), time: Some(time) });
    }

    pub fn mark(&mut self, pos: Position, color: Color, lifetime: f32) {
        self.marks.push(DebugMark { pos, color, lifetime });
    }

    pub fn circle(&mut self, position: Position, radius: f32, color: Color) {
        self.circles.push((position, radius, color));
    }

    pub fn line(
        &mut self,
        start: Position,
        end: Position,
        width: f32,
        z_index: i32,
        color: Color,
    ) {
        self.lines.push(QueuedLine { start, end, width, z_index, color });
    }

    pub fn ray(
        &mut self,
        start: Vec2,
        dir: Vec2,
        width: f32,
        z_index: i32,
        color: Color,
    ) {
        self.line(
            start.as_world(),
            (start + dir).as_world(),
            width,
            z_index,
            color,
        );
    }


    pub fn texture(
        &mut self,
        texture: TextureHandle,
        position: Position,
        z_index: i32,
        color: Color,
        params: DrawTextureParams,
    ) {
        self.textures.push(QueuedTexture {
            texture,
            position,
            z_index,
            color,
            params,
        });
    }

    pub fn text(
        &mut self,
        text: String,
        position: Vec2,
        color: Color,
        size: f32,
    ) {
        self.texts.push((text, position, color, size));
    }

    // #[allow(dead_code)]
    // pub fn prop(
    //     &self,
    //     pos: Vec2,
    //     frame_x: i32,
    //     frame_y: i32,
    //     tint: Color,
    //     flip_x: bool,
    //     flip_y: bool,
    //     rotation: f32,
    // ) {
    //     let xx = pos.x as f32 - 0.5;
    //     let yy = pos.y as f32 - 0.5;
    //
    //     let params = DrawTextureParams {
    //         dest_size: Some(vec2(1.0, 1.0)),
    //         source: Some(Rect::new(
    //             frame_x as f32 * SIZE,
    //             frame_y as f32 * SIZE,
    //             32.0,
    //             32.0,
    //         )),
    //         rotation: rotation.to_radians(),
    //         flip_x,
    //         flip_y,
    //         // flip_x: (),
    //         // flip_y: (),
    //         // pivot: (),
    //         ..Default::default()
    //     };
    //
    //     draw_texture_ex(self.props, xx, yy, tint, params);
    // }
}
