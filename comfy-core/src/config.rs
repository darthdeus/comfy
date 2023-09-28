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
                if *w <= 0 || *h <= 0 =>
            {
                *w = MIN_WINDOW_SIZE;
                *h = MIN_WINDOW_SIZE;
            }
            _ => (),
        }
        self.clone()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GameConfig {
    pub game_name: &'static str,
    pub version: &'static str,

    pub resolution: ResolutionConfig,
    pub min_resolution: ResolutionConfig,

    pub bloom_enabled: bool,
    pub lighting: GlobalLightingParams,
    pub lighting_enabled: bool,

    pub enable_dynamic_camera: bool,

    pub dev: DevConfig,

    pub scroll_speed: f32,

    pub music_enabled: bool,

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
            game_name: "TODO_GAME_NAME",
            version: "TODO_VERSION",

            resolution,
            min_resolution,

            bloom_enabled: false,
            lighting: GlobalLightingParams::default(),
            lighting_enabled: false,

            dev: DevConfig::default(),

            enable_dynamic_camera: false,

            scroll_speed: 7.0,
            music_enabled: false,

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
            #[cfg(any(not(feature = "dev"), not(feature = "ci-release")))]
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
