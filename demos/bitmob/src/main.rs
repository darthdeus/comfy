#![allow(clippy::uninlined_format_args)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::new_without_default)]

pub use embi::*;

mod assets;

define_versions!();

fn main() {
    color_backtrace::install();
    pollster::block_on(run());
}

pub async fn run() {
    cfg_if! {
        if #[cfg(feature = "demo")] {
             let game_name = "BITMOB (DEMO)";
        } else {
             let game_name = "BITMOB";
        }
    };

    let config = GameConfig {
        game_name,
        version: version_str(),

        hide_version: true,
        hide_menu_window: true,
        hide_upgrades: true,
        hide_title: true,

        lighting: GlobalLightingParams {
            ambient_light_intensity: 1.0,

            ..Default::default()
        },

        ..Default::default()
    };


    let game_state = Box::new(
        EngineState::new(
            config,
            Box::new(move |_c| {
                Arc::new(Mutex::new(Game::new()))
            }),
        ), // .with_main_menu_update(Box::new(main_menu_update)),
    );

    crate::assets::load_assets();

    set_main_camera_zoom(30.0);

    run_embi_main_async(game_state).await;
}

pub struct Game {

}

impl Game {
    pub fn new() -> Self {
        Self {}
    }
}

impl GameLoop for Game {
    fn performance_metrics(&self, _world: &mut World, _ui: &mut egui::Ui) {}

    fn enter_game(
        &mut self,
        _world: &mut World,
        _commands: &mut CommandBuffer,
    ) {
    }

    fn exit_game(&mut self, _c: &mut EngineContext) {}

    fn early_update(&mut self, _c: &mut EngineContext) {}

    fn update(&mut self, _c: &mut EngineContext) {}

    fn late_update(&mut self, _c: &mut EngineContext) {}
}
