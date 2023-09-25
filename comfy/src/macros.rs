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
        $crate::define_main!($name, ComfyGame);

        pub struct ComfyGame {
            pub engine: $crate::EngineState,
            pub state: Option<$state>,
        }

        impl ComfyGame {
            pub fn new(engine: $crate::EngineState) -> Self {
                Self { state: None, engine }
            }
        }

        impl GameLoop for ComfyGame {
            fn engine(&mut self) -> &mut EngineState {
                &mut self.engine
            }

            fn update(&mut self) {
                let mut c = self.engine.make_context();

                let state = self.state.get_or_insert_with(|| {
                    let mut state = $state::new(&mut c);
                    $setup(&mut state, &mut c);

                    state
                });

                run_early_update_stages(&mut c);
                $update(state, &mut c);
                run_late_update_stages(&mut c);
            }
        }
    };

    ($name:literal, $setup:ident, $update:ident) => {
        #[doc(hidden)]
        struct ComfyEmptyState;

        impl ComfyEmptyState {
            #[inline]
            #[doc(hidden)]
            pub fn new(_context: &mut $crate::EngineContext) -> Self {
                Self
            }
        }

        #[inline]
        #[doc(hidden)]
        fn _comfy_setup_empty_state(
            _state: &mut ComfyEmptyState,
            context: &mut $crate::EngineContext,
        ) {
            $setup(context)
        }

        #[inline]
        #[doc(hidden)]
        fn _comfy_update_empty_state(
            _state: &mut ComfyEmptyState,
            context: &mut $crate::EngineContext,
        ) {
            $update(context)
        }

        $crate::simple_game! {
            $name,
            ComfyEmptyState,
            _comfy_setup_empty_state,
            _comfy_update_empty_state
        }
    };

    ($name:literal, $update:ident) => {
        #[inline]
        #[doc(hidden)]
        fn _comfy_setup_empty_context(_context: &mut $crate::EngineContext) {}

        simple_game!($name, _comfy_setup_empty_context, $update);
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
