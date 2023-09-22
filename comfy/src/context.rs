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

    pub fn load_sound_from_bytes(&self, name: &str, bytes: &[u8]) {
        ASSETS.borrow_mut().load_sound_from_bytes(name, bytes);
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
        let _span = span!("early_update");

        if is_key_pressed(KeyCode::Backquote) &&
            is_key_down(KeyCode::LCtrl) &&
            is_key_down(KeyCode::LAlt)
        {
            let mut config = self.config.borrow_mut();
            config.dev.show_debug = !config.dev.show_debug;
        }

        // if self.menu.is_active {
        //     main_menu_system(self);
        // }

        if let Some(game_loop) = &mut self.game_loop {
            let game_loop = game_loop.clone();
            game_loop.lock().early_update(self);
        }
    }

    pub fn update(&mut self) {
        let _span = span!("update");

        if let Some(game_loop) = &mut self.game_loop {
            let game_loop = game_loop.clone();
            game_loop.lock().update(self);
        }
    }

    pub fn late_update(&mut self) {
        let _span = span!("late_update");

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

#[derive(Default)]
pub struct CachedImageLoader {
    images: HashMap<String, (egui::TextureHandle, UVec2)>,
}

impl CachedImageLoader {
    pub fn new() -> Self {
        Self { images: HashMap::new() }
    }

    pub fn load_or_err(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> (egui::TextureId, UVec2) {
        self.load(ctx, path).unwrap_or_else(|| self.load(ctx, "error").unwrap())
    }

    pub fn image_or_err(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> egui::TextureId {
        self.cached_load(ctx, path)
            .unwrap_or_else(|| self.cached_load(ctx, "error").unwrap())
    }

    pub fn load(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> Option<(egui::TextureId, UVec2)> {
        // TODO: make cached loader return error id instead of failing
        let mut failed = false;

        if !self.images.contains_key(path) {
            println!("Loading uncached {}", path);

            let texture = texture_id_safe(path).or_else(|| {
                Assets::error_loading_image(path);

                failed = true;

                None
            })?;

            let image = Assets::load_image_data(path, texture)?;
            let (width, height) =
                (image.width() as usize, image.height() as usize);

            let rgba = image.into_rgba8();
            let image_data = rgba.as_raw();

            let egui_image = egui::ColorImage::from_rgba_unmultiplied(
                [width, height],
                image_data,
            );

            let handle = ctx.load_texture(
                path,
                egui_image,
                egui::TextureOptions::LINEAR,
            );

            // let texture_id = ctx.add_image(handle);
            // let tex = ctx.load_texture(path, image.clone(),
            // egui::TextureFilter::Linear);

            self.images.insert(
                path.to_string(),
                (handle, uvec2(width as u32, height as u32)),
            );
            // println!(
            //     "does it now contain path? {}",
            //     self.images.contains_key(path)
            // );
        }

        let (image, size) = self.images.get(path).unwrap();

        Some((image.id(), *size))
    }

    pub fn cached_load(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> Option<egui::TextureId> {
        self.load(ctx, path).map(|x| x.0)
    }
}

pub struct CombatText {
    pub position: Vec2,
    pub text: String,
    pub color: Color,
    pub size: f32,
}

impl CombatText {
    pub fn new(position: Vec2, text: String, color: Color, size: f32) -> Self {
        Self { position, text, color, size }
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

pub fn spawn_combat_text(
    commands: &mut CommandBuffer,
    text: String,
    color: Color,
    size: f32,
    position: Vec2,
) {
    commands.spawn((
        CombatText::new(position, text, color, size),
        DespawnAfter(COMBAT_TEXT_LIFETIME),
    ));
}

pub fn combat_text_system(c: &mut EngineContext) {
    for (_, (combat_text, lifetime)) in
        c.world.borrow_mut().query_mut::<(&mut CombatText, &DespawnAfter)>()
    {
        let progress =
            (COMBAT_TEXT_LIFETIME - lifetime.0) / COMBAT_TEXT_LIFETIME;

        // let dims = measure_text(
        //     &combat_text.text,
        //     Some(c.font),
        //     font_size,
        //     font_scale,
        // );

        let off = 0.5;
        // TODO: re-center
        let pos = combat_text.position + vec2(0.0, 1.0) * progress + off;
        // let pos = pos - vec2(dims.width / 2.0, dims.height / 2.0 + off);

        // let screen_pos = world_to_screen(pos) / egui_scale_factor();

        draw_text_ex(
            &combat_text.text,
            pos,
            // screen_pos.x,
            // screen_pos.y,
            // pos.x,
            // pos.y,
            TextAlign::Center,
            TextParams {
                font: egui::FontId::new(16.0, egui::FontFamily::Proportional),
                color: combat_text.color,
                ..Default::default()
            },
        );
    }
}
