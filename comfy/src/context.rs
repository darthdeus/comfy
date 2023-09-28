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
    pub renderer: &'a mut WgpuRenderer,

    pub draw: &'a RefCell<Draw>,

    pub delta: f32,
    pub frame: u64,

    pub dt_stats: &'a mut MovingStats,
    pub fps_stats: &'a mut MovingStats,
    pub lighting: &'a mut GlobalLightingParams,

    pub meta: &'a mut AnyMap,

    pub egui: &'a egui::Context,
    pub egui_wants_mouse: bool,

    pub cooldowns: &'a mut RefCell<Cooldowns>,
    pub changes: &'a mut RefCell<ChangeTracker>,
    pub notifications: &'a mut RefCell<Notifications>,

    pub config: &'a mut RefCell<GameConfig>,
    pub game_loop: &'a mut Option<Arc<Mutex<dyn GameLoop>>>,

    pub mouse_world: Vec2,
    pub is_paused: &'a mut RefCell<bool>,
    pub show_pause_menu: &'a mut bool,

    pub to_despawn: &'a mut RefCell<Vec<Entity>>,
    pub quit_flag: &'a mut bool,
    pub flags: &'a mut RefCell<HashSet<String>>,

    pub texture_creator: &'a Arc<AtomicRefCell<WgpuTextureCreator>>,
}

impl<'a> EngineContext<'a> {
    pub fn reset_world_and_physics(&mut self) {
        main_camera_mut().center = Vec2::ZERO;
        *self.is_paused.borrow_mut() = false;
        reset_world();
        blood_canvas_reset();
    }

    pub fn load_texture_from_bytes(&self, name: &str, bytes: &[u8]) {
        load_texture_from_engine_bytes(
            &self.renderer.context,
            name,
            bytes,
            &mut self.renderer.textures.lock(),
            wgpu::AddressMode::ClampToEdge,
        );
    }

    pub fn load_texture_from_bytes_ex(
        &self,
        name: &str,
        bytes: &[u8],
        address_mode: wgpu::AddressMode,
    ) {
        load_texture_from_engine_bytes(
            &self.renderer.context,
            name,
            bytes,
            &mut self.renderer.textures.lock(),
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

    pub fn mark(&self, pos: Vec2, color: Color, lifetime: f32) {
        self.draw_mut().mark(pos.as_world(), color, lifetime);
    }

    pub fn despawn(&self, entity: Entity) {
        self.to_despawn.borrow_mut().push(entity);
    }

    pub fn config(&self) -> core::cell::Ref<GameConfig> {
        self.config.borrow()
    }

    pub fn config_mut(&self) -> core::cell::RefMut<GameConfig> {
        self.config.borrow_mut()
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
            &self.renderer.context.device,
            &[&self.renderer.context.texture_layout],
            &self.renderer.config,
            self.renderer.render_texture_format,
            shader.clone(),
        );

        if index == -1 {
            self.renderer.post_processing_effects.borrow_mut().push(effect);
        } else if index >= 0 {
            self.renderer
                .post_processing_effects
                .borrow_mut()
                .insert(index as usize, effect);
        } else {
            panic!("Invalid index = {}, must be -1 or non-negative.", index);
        }

        self.renderer
            .shaders
            .borrow_mut()
            .insert(name.to_string().into(), shader);
    }

    // pub fn early_update(&mut self) {
    //     let _span = span!("context.early_update");
    //
    //     if let Some(game_loop) = &mut self.game_loop {
    //         let game_loop = game_loop.clone();
    //         game_loop.lock().early_update(self);
    //     }
    // }

    // pub fn update(&mut self) {
    //     let _span = span!("context.update");
    //
    //     if let Some(game_loop) = &mut self.game_loop {
    //         let game_loop = game_loop.clone();
    //         game_loop.lock().update(self);
    //     }
    // }
    //
    // pub fn late_update(&mut self) {
    //     let _span = span!("context.late_update");
    //
    //     if let Some(game_loop) = &mut self.game_loop {
    //         let game_loop = game_loop.clone();
    //         game_loop.lock().late_update(self);
    //     }
    // }
}
