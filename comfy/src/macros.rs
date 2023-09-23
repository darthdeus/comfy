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
    ($name:literal, $context:ident, $state:ident, $make_context:ident, $setup:ident, $update:ident) => {
        define_main!($name, ComfyGame);

        pub struct ComfyGame {
            pub engine: EngineState,
            pub state: Option<$state>,
            pub setup_called: bool,
        }

        impl ComfyGame {
            pub fn new(engine: EngineState) -> Self {
                Self { state: None, engine, setup_called: false }
            }
        }

        impl GameLoop for ComfyGame {
            fn update(&mut self) {
                let mut c = self.engine.make_context();

                if self.state.is_none() {
                    self.state = Some(GameState::new(&mut c));
                }

                if let Some(state) = self.state.as_mut() {
                    run_early_update_stages(&mut c);

                    {
                        let mut game_c = $make_context(state, &mut c);

                        if !self.setup_called {
                            self.setup_called = true;

                            $setup(&mut game_c);
                        }

                        $update(&mut game_c);
                    }

                    run_late_update_stages(&mut c);
                }
            }

            fn engine(&mut self) -> &mut EngineState {
                &mut self.engine
            }
        }
    };
}
