use comfy::*;

// As in other state-based example we define a global state object
// for our game.
pub struct MyGame {
    pub x: i32,
    pub y: i32,
}

// Everything interesting happens here.
impl GameLoop for MyGame {
    fn update(&mut self, _c: &mut EngineContext) {
        draw_text(
            &format!(
                "I'm a very advanced example {} + {} = :O",
                self.x, self.y
            ),
            Vec2::ZERO,
            WHITE,
            TextAlign::Center,
        );

        if is_key_pressed(KeyCode::Space) {
            self.y += 1;
        }
    }
}

// -------------------------------------------------------------------
// The following is the `define_main!()` macro used in other examples,
// expanded for extra clarity.
//
// This isn't likely what most users will want, but it shows that
// comfy can be used without any macros or magic.
//
// We currently don't provide a way to return control over the main game loop
// to the user because of how winit's event loop works. Internally when
// `run_comfy_main_async(...)` is called it ends up calling `event_loop.run(...)`
// on winit, which ends up blocking forever.
// -------------------------------------------------------------------

pub fn _comfy_default_config(config: GameConfig) -> GameConfig {
    config
}

pub async fn run() {
    // comfy includes a `define_versions!()` macro that creates a `version_str()`
    // function that returns a version from cargo & git.
    init_game_config(
        "Full Game Loop Example".to_string(),
        "v0.0.1",
        _comfy_default_config,
    );

    let engine = EngineState::new();
    // We can do whatever initialization we want in this case.
    let game = MyGame { x: 2, y: 3 };

    run_comfy_main_async(game, engine).await;
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
