use crate::*;
use bod_core::random::*;

pub static SINGLE_PARTICLES: Lazy<AtomicRefCell<Vec<Particle>>> =
    Lazy::new(|| AtomicRefCell::new(Vec::new()));

#[derive(Copy, Clone, Debug)]
pub enum FadeType {
    Size,
    Alpha,
    FromBlack,
    Both,
    None,
}

pub struct ParticleSystem {
    pub size: Option<Vec2>,
    pub particles: Vec<Particle>,
    pub max_particles: usize,
    pub spawn_rate: Option<f32>,
    pub is_enabled: bool,

    spawn_timer: f32,
    next_particle: usize,
    particle_builder: Box<dyn Fn() -> Particle + Send + Sync>,
    spawn_on_death: bool,
}

impl ParticleSystem {
    fn make_particles(
        max_particles: usize,
        builder: &(dyn Fn() -> Particle + Send + Sync),
    ) -> Vec<Particle> {
        (0..max_particles)
            .map(|_| {
                let mut particle = builder();
                particle.initialize(Vec2::ZERO, None);
                particle.lifetime_current *= random();
                particle
            })
            .collect()
    }

    pub fn with_spawn_rate(
        max_particles: usize,
        spawn_rate: f32,
        particle_builder: impl Fn() -> Particle + Send + Sync + 'static,
    ) -> Self {
        let particle_builder: Box<dyn Fn() -> Particle + Send + Sync> =
            Box::new(particle_builder);

        Self {
            size: None,
            particles: Self::make_particles(max_particles, &particle_builder),
            max_particles,
            spawn_rate: Some(spawn_rate),
            spawn_timer: 0.0,
            next_particle: 0,
            particle_builder,
            spawn_on_death: false,
            is_enabled: true,
        }
    }

    pub fn with_spawn_on_death(
        max_particles: usize,
        particle_builder: impl Fn() -> Particle + Send + Sync + 'static,
    ) -> Self {
        let particle_builder: Box<dyn Fn() -> Particle + Send + Sync> =
            Box::new(particle_builder);

        Self {
            size: None,
            particles: Self::make_particles(max_particles, &particle_builder),
            max_particles,
            spawn_rate: None,
            spawn_timer: 0.0,
            next_particle: 0,
            particle_builder: Box::new(particle_builder),
            spawn_on_death: true,
            is_enabled: true,
        }
    }

    pub fn with_size(self, size: Vec2) -> Self {
        let mut system = Self { size: Some(size), ..self };

        for particle in system.particles.iter_mut() {
            *particle = (system.particle_builder)();
            particle.initialize(Vec2::ZERO, Some(size));
        }

        system
    }

    fn is_particle_inside_box(
        position: Vec2,
        size: Vec2,
        particle: &Particle,
    ) -> bool {
        rect_contains(position, size, particle.position)
    }

    pub fn update(&mut self, position: Vec2, delta: f32) {
        for particle in &mut self.particles {
            particle.update(delta);

            // Spawn a new particle immediately when one dies, if spawn_on_death is enabled
            if self.spawn_on_death &&
                particle.lifetime_current <= 0.0 &&
                self.is_enabled
            {
                *particle = (self.particle_builder)();
                particle.initialize(position, self.size);
            }

            // Check if particle is outside the bounding box and respawn if necessary
            if let Some(size) = self.size {
                if !Self::is_particle_inside_box(position, size, particle) {
                    *particle = (self.particle_builder)();
                    particle.initialize(position, self.size);
                }
            }
        }

        // Update spawn timer and check if it's time to spawn a new particle
        if let Some(spawn_rate) = self.spawn_rate {
            self.spawn_timer += delta;

            while self.spawn_timer >= spawn_rate && self.is_enabled {
                self.spawn_timer -= spawn_rate;

                // Re-initialize the next particle if its lifetime has ended
                let particle = &mut self.particles[self.next_particle];
                if particle.lifetime_current <= 0.0 {
                    *particle = (self.particle_builder)();
                    particle.initialize(position, self.size)
                }

                // Move to the next particle
                self.next_particle =
                    (self.next_particle + 1) % self.max_particles;
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Spritesheet {
    pub rows: usize,
    pub columns: usize,
}

impl Spritesheet {
    pub fn simple(
        rows: usize,
        columns: usize,
        x: usize,
        y: usize,
    ) -> egui::Rect {
        Spritesheet { rows, columns }.sprite(x, y)
    }

    pub fn sprite(self, x: usize, y: usize) -> egui::Rect {
        let step_x = 1.0 / self.columns as f32;
        let step_y = 1.0 / self.rows as f32;

        let start = egui::pos2(step_x * x as f32, step_y * y as f32);

        egui::Rect::from_min_max(start, start + egui::vec2(step_x, step_y))
    }
}

#[derive(Copy, Clone)]
pub enum FadeInDuration {
    Relative(f32),
    Absolute(f32),
    None,
}

#[derive(Clone)]
pub enum TrailRef {
    None,
    Local(Trail),
    System(Index),
}

#[derive(Clone)]
pub struct Particle {
    pub position: Vec2,
    pub offset: Vec2,
    pub rotation: f32,
    pub direction: Vec2,
    pub velocity: f32,
    pub velocity_end: f32,

    pub velocity_curve: fn(f32) -> f32,
    pub size_curve: fn(f32) -> f32,
    pub color_curve: fn(f32) -> f32,

    pub fade_in_duration: FadeInDuration,

    pub z_index: i32,
    pub size: Vec2,

    pub angular_velocity: f32,

    pub start_time: f32,
    pub lifetime_current: f32,
    pub lifetime_max: f32,

    pub color_start: Color,
    pub color_end: Color,

    pub texture: TextureHandle,
    pub source_rect: Option<IRect>,
    pub spritesheet: Option<Spritesheet>,
    pub play_once: bool,

    pub frame: usize,
    pub frame_rate: f32,
    pub animation_timer: f32,

    pub fade_type: FadeType,

    pub blend_mode: BlendMode,

    pub update: Option<fn(&mut Particle)>,
    pub trail: TrailRef,
}

impl Particle {
    pub fn initialize(&mut self, position: Vec2, size: Option<Vec2>) {
        self.position += random_box(position, size.unwrap_or(Vec2::ZERO));
        self.lifetime_current = self.lifetime_max * gen_range(0.9, 1.1);
        self.frame = 0;
        self.animation_timer = 0.0;
        self.update_source_rect();
    }

    pub fn update(&mut self, delta: f32) {
        self.position += self.current_velocity() * delta;
        self.rotation += self.angular_velocity * delta;
        self.lifetime_current -= delta;

        match self.trail {
            TrailRef::Local(ref mut trail) => {
                // TODO: maybe handle pause?
                trail.update(self.position, delta);
                trail.draw_mesh();
            }
            // Updated by the system
            TrailRef::System(_index) => {}
            TrailRef::None => {}
        }

        if let Some(update) = self.update {
            update(self);
        }

        self.update_animation(delta);
    }

    pub fn current_velocity(&self) -> Vec2 {
        let t = 1.0 - self.lifetime_pct();
        self.direction.normalize_or_right() *
            self.velocity *
            (self.velocity_curve)(t)
    }

    // pub fn current_size(&self) -> Vec2 {
    //     let t = self.lifetime_pct();
    //     self.size.lerp(Vec2::ZERO, (self.size_curve)(t))
    // }

    // fn combined_ease(t: f32) -> f32 {
    //     let t = t.min(1.0).max(0.0);
    //     let fade_in = (1.0 - t) * (1.0 - quad_in(t));
    //     let fade_out = t * (1.0 - quad_out(1.0 - t));
    //
    //     fade_in + fade_out
    // }

    // fn combined_ease(t: f32, fade_in_duration: f32) -> f32 {
    //     let t = t.min(1.0).max(0.0);
    //     let fade_in_t = (t / fade_in_duration).min(1.0);
    //     let fade_in = (1.0 - fade_in_t) * (1.0 - quad_in(fade_in_t));
    //     let fade_out = t * (1.0 - quad_out(1.0 - t));
    //
    //     fade_in + fade_out
    // }

    fn combined_ease(
        t: f32,
        lifetime_max: f32,
        fade_in_duration: FadeInDuration,
    ) -> f32 {
        let fade_in_t = match fade_in_duration {
            FadeInDuration::None => 1.0,
            FadeInDuration::Relative(rel_duration) => t / rel_duration,
            FadeInDuration::Absolute(abs_duration) => {
                t * lifetime_max / abs_duration
            }
        };

        let fade_in_t = fade_in_t.min(1.0).max(0.0);
        let fade_in = (1.0 - fade_in_t) * (1.0 - quad_in(fade_in_t));
        let fade_out = t * (1.0 - quad_out(1.0 - t));

        fade_in + fade_out
    }

    // pub fn current_size(&self) -> Vec2 {
    //     let t = self.lifetime_pct();
    //     self.size.lerp(Vec2::ZERO, Self::combined_ease(t))
    // }

    pub fn current_size(&self) -> Vec2 {
        match self.fade_type {
            FadeType::Size | FadeType::Both => {
                let t = self.lifetime_pct();

                self.size.lerp(
                    Vec2::ZERO,
                    Self::combined_ease(
                        t,
                        self.lifetime_max,
                        self.fade_in_duration,
                    ),
                )
            }

            _ => self.size,
        }
    }

    pub fn current_color(&self) -> Color {
        let t = self.lifetime_pct();
        let color =
            self.color_start.lerp(self.color_end, (self.color_curve)(t));

        match self.fade_type {
            FadeType::FromBlack => {
                color.lerp(
                    BLACK,
                    Self::combined_ease(
                        t,
                        self.lifetime_max,
                        self.fade_in_duration,
                    ),
                )
            }
            FadeType::Alpha | FadeType::Both => {
                color.alpha(color.a.lerp(
                    0.0,
                    Self::combined_ease(
                        t,
                        self.lifetime_max,
                        self.fade_in_duration,
                    ),
                ))
                // let t = self.lifetime_pct();
                // let mut result = self.color_start.lerp(
                //     self.color_end,
                //     Self::combined_ease(
                //         t,
                //         self.lifetime_max,
                //         self.fade_in_duration,
                //     ),
                // );
                //
                // result.a = (self.color_curve)(t);
                // result
            }
            FadeType::Size | FadeType::None => color,
        }
    }

    // pub fn current_color(&self) -> Color {
    //     let t = self.lifetime_pct();
    //     let mut result =
    //         self.color_start.lerp(self.color_end, (self.color_curve)(t));
    //
    //     result.a = (self.color_curve)(t);
    //     result
    // }

    pub fn lifetime_pct(&self) -> f32 {
        1.0 - self.lifetime_current / self.lifetime_max
    }

    pub fn update_animation(&mut self, delta: f32) {
        if let Some(spritesheet) = self.spritesheet {
            self.animation_timer += delta;

            while self.animation_timer >= self.frame_rate {
                self.animation_timer -= self.frame_rate;

                let frame_count = spritesheet.rows * spritesheet.columns;

                self.frame += 1;
                if self.frame >= frame_count {
                    self.frame = 0;

                    if self.play_once {
                        self.lifetime_current = 0.0;
                    }
                }

                self.update_source_rect();
            }
        }
    }

    pub fn update_source_rect(&mut self) {
        if let Some(spritesheet) = self.spritesheet {
            let size = Assets::image_size(self.texture).unwrap_or(UVec2::ONE);

            // Update source_rect with the new frame's coordinates
            let sprite_width = size.x as i32 / spritesheet.columns as i32;
            let sprite_height = size.y as i32 / spritesheet.rows as i32;
            let x = (self.frame % spritesheet.columns) as i32 * sprite_width;
            let y = (self.frame / spritesheet.columns) as i32 * sprite_height;

            self.source_rect = Some(IRect::new(
                ivec2(x, y),
                ivec2(sprite_width, sprite_height),
            ));
        }
    }
}

const DEFAULT_EASE: fn(f32) -> f32 = quad_in_out;

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            offset: Vec2::ZERO,
            rotation: 0.0,
            direction: random_dir(),
            velocity: 1.0,
            velocity_end: 0.0,
            angular_velocity: 0.0,
            z_index: 80,

            size: splat(1.0),

            // velocity_curve: |_t| 1.0,
            velocity_curve: quad_in,
            size_curve: DEFAULT_EASE,
            color_curve: DEFAULT_EASE,

            fade_in_duration: FadeInDuration::Absolute(0.1),

            color_start: WHITE,
            color_end: WHITE,

            start_time: get_time() as f32,
            lifetime_current: 0.0,
            lifetime_max: 1.0,

            texture: texture_id("error"),
            source_rect: None,
            spritesheet: None,
            play_once: false,

            frame: 0,
            frame_rate: 1.0 / 60.0,
            animation_timer: 0.0,

            fade_type: FadeType::Size,
            blend_mode: BlendMode::None,
            trail: TrailRef::None,

            update: None,
        }
    }
}

pub fn spawn_particle(mut particle: Particle) {
    assert!(
        particle.lifetime_current == 0.0,
        "lifetime_current is derived, set lifetime_max instead"
    );

    particle.initialize(Vec2::ZERO, None);

    SINGLE_PARTICLES.borrow_mut().push(particle);
}

pub fn spawn_particle_fan(
    num: i32,
    dir: Vec2,
    wiggle_radians: f32,
    velocity_range: Range<f32>,
    map: impl Fn(Particle) -> Particle,
) {
    for _ in 0..num {
        let direction = dir.normalize_or_right().wiggle(wiggle_radians);

        let particle = map(Particle {
            direction,
            velocity: gen_range(velocity_range.start, velocity_range.end),
            ..Default::default()
        });

        spawn_particle(particle);
    }
}
