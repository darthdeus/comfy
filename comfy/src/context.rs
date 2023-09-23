use comfy_wgpu::wgpu::SurfaceConfiguration;

use crate::*;

pub const PAUSE_DESPAWN: &str = "PAUSE_DESPAWN";
pub const PAUSE_PHYSICS: &str = "PAUSE_PHYSICS";

pub const NEW_GAME_FLAG: &str = "NEW_GAME";
pub const EXIT_GAME_FLAG: &str = "EXIT_GAME";

#[derive(Copy, Clone, Debug)]
pub struct RbdSyncSettings {
    pub copy_rotation: bool,
}

impl RbdSyncSettings {
    pub fn new() -> Self {
        Self { copy_rotation: false }
    }

    pub fn copy_rotation(self, copy_rotation: bool) -> Self {
        Self { copy_rotation }
    }
}

pub type ContextFn =
    Box<dyn FnOnce(&mut EngineContext) + Sync + Send + 'static>;

pub struct EngineContext<'a> {
    pub cached_loader: &'a RefCell<CachedImageLoader>,
    pub graphics_context: &'a GraphicsContext,
    pub textures: &'a Arc<Mutex<TextureMap>>,
    pub surface_config: &'a SurfaceConfiguration,
    pub render_texture_format: wgpu::TextureFormat,

    pub draw: &'a RefCell<Draw>,

    pub world: &'a mut Rc<RefCell<World>>,
    pub commands: &'a RefCell<CommandBuffer>,

    pub delta: f32,
    pub frame: u64,

    pub dt_stats: &'a mut MovingStats,
    pub fps_stats: &'a mut MovingStats,
    pub lighting: &'a mut GlobalLightingParams,

    pub meta: &'a mut AnyMap,

    pub egui: &'a egui::Context,
    pub egui_wants_mouse: bool,

    pub cooldowns: &'a RefCell<Cooldowns>,
    pub changes: &'a RefCell<ChangeTracker>,
    pub notifications: &'a RefCell<Notifications>,

    pub config: &'a RefCell<GameConfig>,
    pub game_loop: &'a mut Option<Arc<Mutex<dyn GameLoop>>>,

    pub mouse_world: Vec2,
    pub is_paused: &'a RefCell<bool>,
    pub show_pause_menu: &'a mut bool,

    pub post_processing_effects: &'a RefCell<Vec<PostProcessingEffect>>,
    pub shaders: &'a RefCell<ShaderMap>,

    pub to_despawn: &'a RefCell<Vec<Entity>>,
    pub quit_flag: &'a mut bool,
    pub flags: &'a RefCell<HashSet<String>>,

    pub texture_creator: &'a Arc<AtomicRefCell<WgpuTextureCreator>>,
}

impl<'a> EngineContext<'a> {
    pub fn reset_world_and_physics(&mut self) {
        main_camera_mut().center = Vec2::ZERO;
        *self.is_paused.borrow_mut() = false;
        *self.world = Rc::new(RefCell::new(World::new()));
        blood_canvas_reset();
    }

    pub fn load_texture_from_bytes(
        &self,
        name: &str,
        bytes: &[u8],
        address_mode: wgpu::AddressMode,
    ) {
        load_texture_from_engine_bytes(
            self.graphics_context,
            name,
            bytes,
            &mut self.textures.lock(),
            address_mode,
        );
    }

    pub fn load_sound_from_bytes(
        &self,
        name: &str,
        bytes: &[u8],
        settings: StaticSoundSettings,
    ) {
        ASSETS.borrow_mut().load_sound_from_bytes(name, bytes, settings);
    }

    pub fn load_fonts_from_bytes(&self, fonts: &[(&str, &[u8])]) {
        let mut font_defs = egui::FontDefinitions::default();

        for (name, bytes) in fonts {
            let family_name = name.to_string();

            font_defs.font_data.insert(
                family_name.clone(),
                egui::FontData::from_owned(bytes.to_vec()),
            );

            font_defs.families.insert(
                egui::FontFamily::Name(family_name.clone().into()),
                vec![family_name],
            );
        }

        self.egui.set_fonts(font_defs);
    }

    pub fn commands(&self) -> core::cell::RefMut<CommandBuffer> {
        self.commands.borrow_mut()
    }

    pub fn mark(&self, pos: Vec2, color: Color, lifetime: f32) {
        self.draw_mut().mark(pos.as_world(), color, lifetime);
    }

    pub fn despawn(&self, entity: Entity) {
        self.to_despawn.borrow_mut().push(entity);
    }

    pub fn query() {
        todo!();
    }

    pub fn world(&self) -> core::cell::Ref<World> {
        self.world.borrow()
    }

    pub fn world_mut(&self) -> core::cell::RefMut<World> {
        self.world.borrow_mut()
    }

    pub fn config(&self) -> core::cell::Ref<GameConfig> {
        self.config.borrow()
    }

    pub fn config_mut(&self) -> core::cell::RefMut<GameConfig> {
        self.config.borrow_mut()
    }

    pub fn cooldowns(&self) -> core::cell::RefMut<Cooldowns> {
        self.cooldowns.borrow_mut()
    }

    pub fn draw(&self) -> core::cell::Ref<Draw> {
        self.draw.borrow()
    }

    pub fn draw_mut(&self) -> core::cell::RefMut<Draw> {
        self.draw.borrow_mut()
    }

    pub fn insert_post_processing_effect(
        &self,
        index: i32,
        name: &str,
        shader: Shader,
    ) {
        let effect = PostProcessingEffect::new(
            name.to_string(),
            &self.graphics_context.device,
            &[&self.graphics_context.texture_layout],
            self.surface_config,
            self.render_texture_format,
            shader.clone(),
        );

        if index == -1 {
            self.post_processing_effects.borrow_mut().push(effect);
        } else if index >= 0 {
            self.post_processing_effects
                .borrow_mut()
                .insert(index as usize, effect);
        } else {
            panic!("Invalid index = {}, must be -1 or non-negative.", index);
        }

        self.shaders
            .borrow_mut()
            .insert(name.to_string().into(), shader.clone());
    }

    pub fn early_update(&mut self) {
        let _span = span!("context.early_update");


        if let Some(game_loop) = &mut self.game_loop {
            let game_loop = game_loop.clone();
            game_loop.lock().early_update(self);
        }

        let mut transforms = HashMap::new();

        for (entity, transform) in self.world_mut().query_mut::<&Transform>() {
            transforms.insert(entity, *transform);
        }

        for (_, transform) in self.world().query::<&mut Transform>().iter() {
            let parent = if let Some(parent) = transform.parent {
                transforms
                    .get(&parent)
                    .cloned()
                    .unwrap_or(Transform::position(Vec2::ZERO))
            } else {
                Transform::position(Vec2::ZERO)
            };

            let combined = transform.compose_with_parent(&parent);

            transform.abs_position = combined.position;
            transform.abs_rotation = combined.rotation;
            transform.abs_scale = combined.scale;
        }

        for (_, (transform, light)) in
            self.world_mut().query_mut::<(&Transform, &PointLight)>()
        {
            draw_light(Light::simple(
                transform.position,
                light.radius * light.radius_mod,
                light.strength * light.strength_mod,
            ))
        }

        if !*self.is_paused.borrow() &&
            !self.flags.borrow().contains(PAUSE_PHYSICS)
        {
            self.cooldowns.borrow_mut().tick(self.delta);
            self.notifications.borrow_mut().tick(self.delta);
        }

        timings_add_value("delta", delta());

        let mut call_queue = vec![];

        if !*self.is_paused.borrow() {
            for (entity, sprite) in
                self.world().query::<&mut AnimatedSprite>().iter()
            {
                if sprite.state.update_and_finished(self.delta) {
                    self.commands().despawn(entity);

                    // TODO: maybe not needed? replace with option
                    let mut temp: ContextFn = Box::new(|_| {});
                    std::mem::swap(&mut sprite.on_finished, &mut temp);

                    call_queue.push(temp);
                }
            }
        }

        for call in call_queue.drain(..) {
            call(self);
        }
    }

    pub fn update(&mut self) {
        let _span = span!("context.update");

        if let Some(game_loop) = &mut self.game_loop {
            let game_loop = game_loop.clone();
            game_loop.lock().update(self);
        }
    }

    pub fn late_update(&mut self) {
        let _span = span!("context.late_update");

        let delta = delta();

        if let Some(game_loop) = &mut self.game_loop {
            let game_loop = game_loop.clone();
            game_loop.lock().late_update(self);
        }

        self.draw.borrow_mut().marks.retain_mut(|mark| {
            mark.lifetime -= delta;
            mark.lifetime > 0.0
        });

        for mark in self.draw.borrow().marks.iter() {
            draw_circle_z(
                mark.pos.to_world(),
                0.1,
                mark.color,
                90,
                TextureParams {
                    blend_mode: BlendMode::Alpha,
                    ..Default::default()
                },
            );
        }

        let is_paused = *self.is_paused.borrow() ||
            self.flags.borrow_mut().contains(PAUSE_DESPAWN);

        if !is_paused {
            for (entity, despawn) in
                self.world.borrow_mut().query_mut::<&mut DespawnAfter>()
            {
                despawn.0 -= delta;

                if despawn.0 <= 0.0 {
                    self.to_despawn.borrow_mut().push(entity);
                }
            }
        }

        main_camera_mut().update(delta);
        self.commands().run_on(&mut self.world.borrow_mut());
        self.world.borrow_mut().flush();
    }
}


pub fn count_to_color(count: i32) -> Color {
    match count {
        0 => WHITE,
        1 => BLUE,
        2 => GREEN,
        3 => RED,
        4 => PURPLE,
        _ => YELLOW,
    }
}

