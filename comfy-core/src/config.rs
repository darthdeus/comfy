use crate::*;

pub const COMBAT_TEXT_LIFETIME: f32 = 0.4;

#[derive(Copy, Clone, Debug)]
pub enum ResolutionConfig {
    Physical(u32, u32),
    Logical(u32, u32),
}

impl ResolutionConfig {
    pub fn width(&self) -> u32 {
        match self {
            Self::Physical(w, _) => *w,
            Self::Logical(w, _) => *w,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Self::Physical(_, h) => *h,
            Self::Logical(_, h) => *h,
        }
    }

    pub fn ensure_non_zero(&mut self) -> ResolutionConfig {
        const MIN_WINDOW_SIZE: u32 = 1;
        match self {
            ResolutionConfig::Physical(w, h) |
            ResolutionConfig::Logical(w, h)
                if *w == 0 || *h == 0 =>
            {
                *w = MIN_WINDOW_SIZE;
                *h = MIN_WINDOW_SIZE;
            }
            _ => (),
        }

        *self
    }
}

static GAME_CONFIG: OnceCell<AtomicRefCell<GameConfig>> = OnceCell::new();

pub fn init_game_config(
    game_name: String,
    version: &'static str,
    config_fn: fn(GameConfig) -> GameConfig,
) {
    GAME_CONFIG
        .set(AtomicRefCell::new(config_fn(GameConfig {
            game_name,
            version,
            ..Default::default()
        })))
        .expect(
            "init_game_config() should only be called once by comfy itself",
        );
}

pub fn game_config() -> AtomicRef<'static, GameConfig> {
    GAME_CONFIG
        .get()
        .expect("game_config() must be called after comfy main runs")
        .borrow()
}

pub fn game_config_mut() -> AtomicRefMut<'static, GameConfig> {
    GAME_CONFIG
        .get()
        .expect("game_config() must be called after comfy main runs")
        .borrow_mut()
}

#[derive(Clone, Debug)]
pub struct GameConfig {
    pub game_name: String,
    pub version: &'static str,

    pub resolution: ResolutionConfig,
    pub min_resolution: ResolutionConfig,

    pub target_framerate: u32,
    pub vsync_enabled: bool,

    pub bloom_enabled: bool,
    pub lighting: GlobalLightingParams,
    pub lighting_enabled: bool,

    pub enable_dynamic_camera: bool,

    pub dev: DevConfig,

    pub scroll_speed: f32,

    pub music_enabled: bool,
    pub blood_canvas_z: i32,

    pub show_combat_text: bool,
    pub spawn_exp: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let resolution = ResolutionConfig::Logical(1106, 526);
        #[cfg(not(target_arch = "wasm32"))]
        let resolution = ResolutionConfig::Physical(1920, 1080);

        #[cfg(target_arch = "wasm32")]
        let min_resolution = ResolutionConfig::Logical(1, 1);
        #[cfg(not(target_arch = "wasm32"))]
        let min_resolution = ResolutionConfig::Physical(1, 1);

        Self {
            game_name: "TODO_GAME_NAME".to_string(),
            version: "TODO_VERSION",

            resolution,
            min_resolution,

            target_framerate: 60,
            vsync_enabled: true,

            bloom_enabled: false,
            lighting: GlobalLightingParams::default(),
            lighting_enabled: false,

            dev: DevConfig::default(),

            enable_dynamic_camera: false,

            scroll_speed: 7.0,
            music_enabled: false,
            blood_canvas_z: 4,

            show_combat_text: true,
            spawn_exp: true,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DevConfig {
    pub show_lighting_config: bool,
    pub show_buffers: bool,
    pub show_fps: bool,
    pub show_editor: bool,

    pub show_tiktok_overlay: bool,

    pub log_collisions: bool,

    pub show_ai_target: bool,
    pub show_linear_acc_target: bool,
    pub show_angular_acc_target: bool,

    pub draw_colliders: bool,
    pub draw_collision_marks: bool,

    pub show_debug_bullets: bool,

    pub orig_props: bool,

    pub collider_outlines: bool,

    pub show_debug: bool,

    pub recording_mode: RecordingMode,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecordingMode {
    None,
    Tiktok,
    Landscape,
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            show_lighting_config: false,
            show_buffers: false,
            show_editor: false,

            log_collisions: false,

            show_ai_target: false,
            show_linear_acc_target: false,
            show_angular_acc_target: false,
            show_tiktok_overlay: false,

            show_debug_bullets: false,

            #[cfg(feature = "ci-release")]
            show_fps: false,
            #[cfg(feature = "dev")]
            show_fps: true,
            #[cfg(all(not(feature = "dev"), not(feature = "ci-release")))]
            show_fps: false,

            draw_colliders: false,
            draw_collision_marks: false,

            collider_outlines: false,
            orig_props: true,
            show_debug: false,

            recording_mode: RecordingMode::Landscape,
        }
    }
}
