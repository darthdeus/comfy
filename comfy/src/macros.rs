#[macro_export]
macro_rules! define_main {
    ($name:literal) => {
        define_versions!();

        pub async fn run() {
            let config = GameConfig {
                game_name: $name,
                version: version_str(),
                ..Default::default()
            };

            let engine_state = EngineState::new(
                config,
                Box::new(move |_c| Arc::new(Mutex::new(Game::new()))),
            );

            run_comfy_main_async(engine_state).await;
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
                    $update(state, c);
                }
            }
        }
    };

    ($name:literal, $setup:ident, $update:ident) => {
        define_main!($name);

        pub struct EmptyGameState {}

        impl EmptyGameState {
            pub fn new(_c: &mut EngineContext) -> Self {
                Self {}
            }
        }

        pub struct Game {
            pub state: Option<EmptyGameState>,
        }

        impl Game {
            pub fn new() -> Self {
                Self { state: None }
            }
        }

        impl GameLoop for Game {
            fn update(&mut self, c: &mut EngineContext) {
                if self.state.is_none() {
                    self.state = Some(EmptyGameState::new(c));
                    $setup(c);
                }

                if let Some(state) = self.state.as_mut() {
                    $update(c);
                }
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
