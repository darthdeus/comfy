use crate::*;

pub const PAUSE_DESPAWN: &str = "PAUSE_DESPAWN";
pub const PAUSE_PHYSICS: &str = "PAUSE_PHYSICS";

pub const NEW_GAME_FLAG: &str = "NEW_GAME";
pub const EXIT_GAME_FLAG: &str = "EXIT_GAME";

pub static TO_DESPAWN_QUEUE: AtomicRefCell<Vec<Entity>> =
    AtomicRefCell::new(Vec::new());

/// Queue up an entity into a global despawn queue.
///
/// This is useful for when you want to despawn an entity from
/// single place in the code at some later stage.
///
/// Use `take_to_despawn` to get the list of entities to despawn
/// and clear the despawn queue.
pub fn despawn(entity: Entity) {
    TO_DESPAWN_QUEUE.borrow_mut().push(entity);
}

/// Returns a list of entities to despawn.
///
/// This consumes the existing queue and replaces it with an empty one.
pub fn take_to_despawn() -> Vec<Entity> {
    std::mem::take(&mut TO_DESPAWN_QUEUE.borrow_mut())
}

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
    pub renderer: &'a mut WgpuRenderer,

    pub draw: &'a RefCell<Draw>,

    pub delta: f32,
    pub frame: u64,

    pub dt_stats: &'a mut MovingStats,
    pub fps_stats: &'a mut MovingStats,

    pub meta: &'a mut AnyMap,

    pub is_paused: &'a mut RefCell<bool>,
    pub show_pause_menu: &'a mut bool,

    pub quit_flag: &'a mut bool,
    pub flags: &'a mut RefCell<HashSet<String>>,

    // TODO: remove this, can be passed through GraphicsContext or WgpuRenderer
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

        egui().set_fonts(font_defs);
    }

    pub fn mark(&self, pos: Vec2, color: Color, lifetime: f32) {
        self.draw_mut().mark(pos.as_world(), color, lifetime);
    }

    pub fn draw(&self) -> core::cell::Ref<Draw> {
        self.draw.borrow()
    }

    pub fn draw_mut(&self) -> core::cell::RefMut<Draw> {
        self.draw.borrow_mut()
    }
}

pub fn load_sound_from_bytes(
    name: &str,
    bytes: &[u8],
    settings: StaticSoundSettings,
) {
    ASSETS.borrow_mut().load_sound_from_bytes(name, bytes, settings);
}
