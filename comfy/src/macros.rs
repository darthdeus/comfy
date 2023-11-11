#[macro_export]
macro_rules! define_main {
    ($name:literal, $game:ident $(,)?) => {
        #[inline]
        #[doc(hidden)]
        pub fn _comfy_default_config(
            config: $crate::GameConfig,
        ) -> $crate::GameConfig {
            config
        }

        define_main!($name, $game, _comfy_default_config);
    };

    ($name:literal, $game:ident, $config:ident $(,)?) => {
        $crate::define_versions!();

        pub async fn run() {
            $crate::init_game_config($name.to_string(), version_str(), $config);

            let mut engine = $crate::EngineState::new();
            let game = $game::new(&mut engine);

            $crate::run_comfy_main_async(game, engine).await;
        }

        fn main() {
            #[cfg(feature = "color-backtrace")]
            $crate::color_backtrace::install();

            #[cfg(not(target_arch = "wasm32"))]
            {
                $crate::pollster::block_on(run());
            }

            #[cfg(target_arch = "wasm32")]
            {
                $crate::wasm_bindgen_futures::spawn_local(run());
            }
        }
    };
}

#[macro_export]
macro_rules! simple_game {
    ($name:literal, $state:ident, $config:ident, $setup:ident, $update:ident $(,)?) => {
        pub struct ComfyGame {
            pub state: $state,
            pub setup_called: bool,
        }

        impl GameLoop for ComfyGame {
            fn new(c: &mut $crate::EngineState) -> Self
            where Self: Sized {
                let state = $state::new(c);
                Self { state, setup_called: false }
            }

            fn update(&mut self, c: &mut $crate::EngineContext) {
                if !self.setup_called {
                    $setup(&mut self.state, c);
                    self.setup_called = true;
                }

                $update(&mut self.state, c);
            }
        }

        $crate::comfy_game! {
            $name,
            ComfyGame,
            $config,
        }
    };

    ($name:literal, $state:ident, $setup:ident, $update:ident $(,)?) => {
        #[inline]
        #[doc(hidden)]
        pub fn _comfy_default_config(
            config: $crate::GameConfig,
        ) -> $crate::GameConfig {
            config
        }

        $crate::simple_game! {
            $name,
            $state,
            _comfy_default_config,
            $setup,
            $update,
        }
    };

    ($name:literal, $setup:ident, $update:ident $(,)?) => {
        #[doc(hidden)]
        pub struct ComfyEmptyState;

        impl ComfyEmptyState {
            #[inline]
            #[must_use]
            #[doc(hidden)]
            pub fn new(_context: &mut $crate::EngineState) -> Self {
                Self
            }
        }

        #[inline]
        #[doc(hidden)]
        fn _comfy_setup_empty_state(
            _state: &mut ComfyEmptyState,
            context: &mut $crate::EngineContext<'_>,
        ) {
            $setup(context)
        }

        #[inline]
        #[doc(hidden)]
        fn _comfy_update_empty_state(
            _state: &mut ComfyEmptyState,
            context: &mut $crate::EngineContext<'_>,
        ) {
            $update(context)
        }

        $crate::simple_game! {
            $name,
            ComfyEmptyState,
            _comfy_setup_empty_state,
            _comfy_update_empty_state,
        }
    };

    ($name:literal, $update:ident $(,)?) => {
        #[inline]
        #[doc(hidden)]
        fn _comfy_setup_empty_context(
            _context: &mut $crate::EngineContext<'_>,
        ) {
        }

        simple_game!($name, _comfy_setup_empty_context, $update);
    };
}

#[macro_export]
macro_rules! comfy_game {
    ($name:literal, $game:ident, $config:ident $(,)?) => {
        $crate::define_main!($name, $game, $config);
    };

    ($name:literal, $game:ident) => {
        #[inline]
        #[doc(hidden)]
        pub fn _comfy_default_config(
            config: $crate::GameConfig,
        ) -> $crate::GameConfig {
            config
        }

        comfy_game!($name, $game, _comfy_default_config);
    };
}

// #[macro_export]
// macro_rules! comfy_game {
//     ($name:literal, $context:ident, $state:ident, $make_context:ident, $config:ident, $setup:ident, $update:ident $(,)?) => {
//         $crate::define_main!($name, __ComfyGame, $config);
//
//         pub struct __ComfyGame {
//             pub engine: $crate::EngineState,
//             pub state: Option<$state>,
//         }
//
//         impl __ComfyGame {
//             #[inline]
//             #[must_use]
//             pub fn new(engine: $crate::EngineState) -> Self {
//                 Self { state: None, engine }
//             }
//         }
//
//         impl $crate::GameLoop for __ComfyGame {
//             fn update(&mut self) {
//                 let mut c = self.engine.make_context();
//
//                 $crate::run_early_update_stages(&mut c);
//
//                 let mut game_c: $context = match self.state.as_mut() {
//                     Some(state) => $make_context(state, &mut c),
//                     None => {
//                         #[allow(clippy::unnecessary_mut_passed)]
//                         let state: $state = $state::new(&mut c);
//                         let state = self.state.insert(state);
//                         let mut game_c = $make_context(state, &mut c);
//                         $setup(&mut game_c);
//                         game_c
//                     }
//                 };
//
//                 $update(&mut game_c);
//
//                 $crate::run_late_update_stages(&mut c, $crate::delta());
//             }
//
//             #[inline]
//             #[must_use]
//             fn engine(&mut self) -> &mut $crate::EngineState {
//                 &mut self.engine
//             }
//         }
//     };
//
//     ($name:literal, $context:ident, $state:ident, $make_context:ident, $setup:ident, $update:ident $(,)?) => {
//         #[inline]
//         #[doc(hidden)]
//         pub fn _comfy_default_config(
//             config: $crate::GameConfig,
//         ) -> $crate::GameConfig {
//             config
//         }
//
//         $crate::comfy_game! {
//             $name,
//             $context,
//             $state,
//             $make_context,
//             _comfy_default_config,
//             $setup,
//             $update,
//         }
//     };
// }
