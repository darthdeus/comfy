#[macro_export]
macro_rules! define_main {
    ($name:literal, $game:ident) => {
        define_versions!();

        pub async fn run() {
            let config = GameConfig {
                game_name: $name,
                version: version_str(),
                ..Default::default()
            };

            let engine = EngineState::new(config);
            let game = $game::new(engine);

            run_comfy_main_async(game).await;
        }

        fn main() {
            #[cfg(feature = "color-backtrace")]
            color_backtrace::install();

            #[cfg(not(target_arch = "wasm32"))]
            {
                pollster::block_on(run());
            }

            #[cfg(target_arch = "wasm32")]
            {
                wasm_bindgen_futures::spawn_local(run());
            }
        }
    };
}

#[macro_export]
macro_rules! simple_game {
    ($name:literal, $state:ident, $setup:ident, $update:ident) => {
        define_main!($name, ComfyGame);

        pub struct ComfyGame {
            pub engine: EngineState,
            pub state: Option<$state>,
        }

        impl ComfyGame {
            pub fn new(engine: EngineState) -> Self {
                Self { state: None, engine }
            }
        }

        impl GameLoop for ComfyGame {
            fn engine(&mut self) -> &mut EngineState {
                &mut self.engine
            }

            fn update(&mut self) {
                let mut c = self.engine.make_context();

                if self.state.is_none() {
                    let mut state = $state::new(&mut c);
                    $setup(&mut state, &mut c);

                    self.state = Some(state);
                }

                if let Some(state) = self.state.as_mut() {
                    run_early_update_stages(&mut c);
                    $update(state, &mut c);
                    run_late_update_stages(&mut c);
                }
            }
        }
    };

    ($name:literal, $setup:ident, $update:ident) => {
        define_main!($name, ComfyGame);

        pub struct ComfyGame {
            pub engine: EngineState,
            pub setup_done: bool,
        }

        impl ComfyGame {
            pub fn new(engine: EngineState) -> Self {
                Self { engine, setup_done: false }
            }
        }

        impl GameLoop for ComfyGame {
            fn engine(&mut self) -> &mut EngineState {
                &mut self.engine
            }

            fn update(&mut self) {
                let mut c = self.engine.make_context();

                if !self.setup_done {
                    self.setup_done = true;
                    $setup(&mut c);
                }

                run_early_update_stages(&mut c);
                $update(&mut c);
                run_late_update_stages(&mut c);
            }
        }
    };

    ($name:literal, $update:ident) => {
        fn setup(_c: &mut EngineContext) {}

        simple_game!($name, setup, $update);
    };
}

#[macro_export]
macro_rules! comfy_game {
    ($name:literal, $make_context:ident, $context:ident, $state:ident, $setup:ident, $update:ident) => {
        define_main!($name);

        pub struct Game {
            pub state: Option<$state>,
        }

        impl Game {
            pub fn new() -> Self {
                Self { state: None }
            }
        }

        impl GameLoop for Game {
            fn update(&mut self, c: &mut EngineContext) {
                if self.state.is_none() {
                    let mut state = $state::new(c);
                    $setup(&mut state, c);

                    self.state = Some(state);
                }

                if let Some(state) = self.state.as_mut() {
                    let mut game_c = make_context(state, c);
                    $update(&mut game_c);
                }
            }
        }
    };
}
