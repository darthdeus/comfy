#[macro_export]
macro_rules! define_main {
    ($run:ident) => {
        define_versions!();

        fn main() {
            #[cfg(feature = "color-backtrace")]
            color_backtrace::install();

            #[cfg(not(target_arch = "wasm32"))]
            {
                pollster::block_on($run());
            }

            #[cfg(target_arch = "wasm32")]
            {
                wasm_bindgen_futures::spawn_local($run());
            }
        }
    };
}

#[macro_export]
macro_rules! simple_game {
    ($name:literal, $setup:ident, $update:ident) => {
        define_main!(run);

        pub async fn run() {
            let config = GameConfig {
                game_name: $name,
                version: version_str(),
                ..Default::default()
            };

            let game_state = Box::new(EngineState::new(
                config,
                Box::new(move |_c| Arc::new(Mutex::new(Game::new()))),
            ));

            run_comfy_main_async(game_state).await;
        }

        pub struct GameState {}

        impl GameState {
            pub fn new(_c: &mut EngineContext) -> Self {
                Self {}
            }
        }

        pub struct Game {
            pub state: Option<GameState>,
        }

        impl Game {
            pub fn new() -> Self {
                Self { state: None }
            }
        }

        impl GameLoop for Game {
            fn performance_metrics(
                &self,
                _world: &mut World,
                _ui: &mut egui::Ui,
            ) {
            }

            fn early_update(&mut self, _c: &mut EngineContext) {}

            fn update(&mut self, c: &mut EngineContext) {
                if self.state.is_none() {
                    self.state = Some(GameState::new(c));
                    $setup(c);
                }

                if let Some(state) = self.state.as_mut() {
                    game_update(state, c);
                }
            }

            fn late_update(&mut self, _c: &mut EngineContext) {}
        }

        fn game_update(_state: &GameState, c: &mut EngineContext) {
            $update(c);
        }
    };

    ($name:literal, $update:ident) => {
        fn setup(_c: &mut EngineContext) {}

        simple_game!($name, setup, $update);
    };
}
