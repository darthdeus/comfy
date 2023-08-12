use crate::*;

#[derive(Clone, Debug)]
pub enum AnimationSource {
    Files {
        prefix: Cow<'static, str>,
        frames: i32,
    },
    Atlas {
        name: Cow<'static, str>,
        offset: IVec2,
        step: IVec2,
        size: IVec2,
        frames: i32,
    },
    Spritesheet {
        name: Cow<'static, str>,
        spritesheet: Spritesheet,
    },
}

impl AnimationSource {
    pub fn frames(&self) -> i32 {
        match self {
            AnimationSource::Files { frames, .. } => *frames,
            AnimationSource::Atlas { frames, .. } => *frames,
            AnimationSource::Spritesheet { spritesheet, .. } => {
                (spritesheet.rows * spritesheet.columns) as i32
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnimationState {
    pub source: AnimationSource,
    pub interval: f32,
    pub looping: bool,
    pub timer: f32,
    pub current_frame: i32,
}

impl AnimationState {
    pub fn new(source: AnimationSource, time: f32, looping: bool) -> Self {
        Self {
            looping,
            interval: time / source.frames() as f32,
            timer: 0.0,
            current_frame: 0,
            source,
        }
    }

    pub fn with_timer(self, timer: f32) -> Self {
        Self { timer, ..self }
    }

    pub fn progress(&self) -> f32 {
        self.timer / (self.interval * self.source.frames() as f32)
    }

    pub fn update_and_finished(&mut self, delta: f32) -> bool {
        let mut should_despawn = false;

        self.timer += delta;

        let idx = (self.timer / self.interval) as i32;

        if idx >= self.source.frames() && !self.looping {
            should_despawn = true;
        }

        self.current_frame = idx % self.source.frames();

        should_despawn
    }

    pub fn current_rect(&self) -> (Cow<'static, str>, Option<IRect>) {
        match self.source {
            AnimationSource::Files { ref prefix, .. } => {
                (
                    Into::<Cow<'static, str>>::into(format!(
                        "{}{}",
                        prefix, self.current_frame
                    )),
                    None,
                )
            }
            AnimationSource::Atlas { ref name, offset, step, size, .. } => {
                (
                    name.clone(),
                    Some(IRect::new(offset + step * self.current_frame, size)),
                )
            }
            AnimationSource::Spritesheet { ref name, spritesheet } => {
                let image_size = Assets::image_size(texture_id(name))
                    .unwrap_or_else(|| {
                        error!("failed to get size for {name}");
                        uvec2(64, 64)
                    })
                    .as_ivec2();

                let size = ivec2(
                    image_size.x / spritesheet.columns as i32,
                    image_size.y / spritesheet.rows as i32,
                );

                let row = self.current_frame / spritesheet.columns as i32;
                let col = self.current_frame % spritesheet.columns as i32;

                let offset = ivec2(col, row) * size;

                let rect = IRect::new(offset, size);


                (name.clone(), Some(rect))
            }
        }
    }
}

pub struct AnimatedSprite {
    pub animations: HashMap<String, AnimationSource>,

    pub state: AnimationState,
    pub z_index: i32,
    pub size: Vec2,
    pub color: Color,

    pub flip_x: bool,
    pub flip_y: bool,

    pub blend_mode: BlendMode,

    pub offset: Vec2,

    pub on_finished: ContextFn,
}

impl Default for AnimatedSprite {
    fn default() -> Self {
        Self {
            animations: Default::default(),
            state: AnimationState {
                source: AnimationSource::Atlas {
                    name: "1px".into(),
                    offset: IVec2::ZERO,
                    step: IVec2::ZERO,
                    size: ivec2(16, 16),
                    frames: 1,
                },
                interval: 0.2,
                looping: true,
                timer: 0.0,
                current_frame: 0,
            },

            z_index: 10,
            size: splat(1.0),
            color: WHITE,
            flip_x: false,
            flip_y: false,
            blend_mode: BlendMode::None,
            offset: Vec2::ZERO,
            on_finished: Box::new(|_| {}),
        }
    }
}

impl AnimatedSprite {
    pub fn from_files(
        prefix: impl Into<Cow<'static, str>>,
        frames: i32,
        interval: f32,
        looping: bool,
        z_index: i32,
        size: Vec2,
        color: Color,
        offset: Vec2,
        on_finished: ContextFn,
    ) -> Self {
        Self {
            animations: HashMap::default(),
            state: AnimationState {
                source: AnimationSource::Files {
                    prefix: prefix.into(),
                    frames,
                },
                interval,
                looping,
                timer: 0.0,
                current_frame: 0,
            },

            z_index,
            size,
            color,

            flip_x: false,
            flip_y: false,

            blend_mode: BlendMode::None,

            offset,

            on_finished,
        }
    }

    pub fn spritesheet(
        name: impl Into<Cow<'static, str>>,
        spritesheet: Spritesheet,
        interval: f32,
        looping: bool,
        z_index: i32,
        world_size: Vec2,
        color: Color,
        px_offset: Vec2,
        on_finished: ContextFn,
    ) -> Self {
        Self {
            animations: HashMap::default(),
            state: AnimationState {
                source: AnimationSource::Spritesheet {
                    name: name.into(),
                    spritesheet,
                },
                interval,
                looping,
                timer: 0.0,
                current_frame: 0,
            },

            z_index,
            size: world_size,
            color,

            flip_x: false,
            flip_y: false,

            blend_mode: BlendMode::None,

            offset: px_offset,

            on_finished,
            // on_finished,
            // on_finished_meta: Arc::new(Mutex::new(None as Option<()>)),
        }
    }

    pub fn atlas(
        name: impl Into<Cow<'static, str>>,
        offset: IVec2,
        step: IVec2,
        sprite_size: IVec2,
        frames: i32,
        interval: f32,
        looping: bool,
        z_index: i32,
        world_size: Vec2,
        color: Color,
        px_offset: Vec2,
        on_finished: ContextFn,
    ) -> Self {
        Self {
            animations: HashMap::default(),
            state: AnimationState {
                source: AnimationSource::Atlas {
                    name: name.into(),
                    offset,
                    step,
                    size: sprite_size,
                    frames,
                },
                interval,
                looping,
                timer: 0.0,
                current_frame: 0,
            },

            z_index,
            size: world_size,
            color,

            flip_x: false,
            flip_y: false,

            blend_mode: BlendMode::None,

            offset: px_offset,

            on_finished,
            // on_finished,
            // on_finished_meta: Arc::new(Mutex::new(None as Option<()>)),
        }
    }

    pub fn with_blend_mode(self, blend_mode: BlendMode) -> Self {
        Self { blend_mode, ..self }
    }

    pub fn to_quad_draw(&self, transform: &Transform) -> QuadDraw {
        let (texture, source_rect) = self.state.current_rect();

        QuadDraw {
            transform: *transform,
            texture: texture_id(&texture),
            z_index: self.z_index,
            color: self.color,
            blend_mode: self.blend_mode,
            dest_size: self.size * transform.scale,
            source_rect,
            flip_x: self.flip_x,
            flip_y: self.flip_y,
        }
    }
}

pub struct AnimatedSpriteBuilder {}
