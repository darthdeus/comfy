use comfy::*;

// As in other state-based example we define a global state object
// for our game.
pub struct GameState {
    pub x: i32,
    pub y: i32,
}

// This state can be initialized from the engine's context.
impl GameState {
    pub fn new(_c: &mut EngineContext) -> Self {
        Self { x: 5, y: 2 }
    }
}

// We also define our own `GameContext` which can be later
// passed all throughout the game's systems.
//
// Unfortunately this does require a fair bit of boilerplate,
// especially the "need to mirror everything in state", but
// the boilerplate tends to pay off later in gameplay code.
//
// One can start simple and just embed `state: &'a mut GameState`,
// although game code will become a lot nicer when individual fields
// are exposed.
pub struct GameContext<'a, 'b: 'a> {
    pub x: &'a mut i32,
    pub y: &'a mut i32,
    pub engine: &'a mut EngineContext<'b>,
}

// Our game owns the `EngineState`, as well as `GameState`. In order
// to allow initialization of `GameState` from `EngineContext` it has
// to be `Option<T>` and initialized later.
pub struct ComfyGame {
    pub engine: EngineState,
    pub state: Option<GameState>,
}

// Necessary boilerplate.
impl ComfyGame {
    pub fn new(engine: EngineState) -> Self {
        Self { state: None, engine }
    }
}

// Everything interesting happens here.
impl GameLoop for ComfyGame {
    fn update(&mut self) {
        // All internal engine code expect an `EngineContext`.
        let mut c = self.engine.make_context();

        // State initialization using `EngineContext`. While this could
        // be simplified if our state is simple, doing it like this
        // has the only downside of a few extra lines of code, and could
        // save some headaches later.
        if self.state.is_none() {
            let mut state = GameState::new(&mut c);
            setup(&mut state, &mut c);

            self.state = Some(state);
        }

        // Now we just run our regular update.
        if let Some(state) = self.state.as_mut() {
            // Right now engine stages have to be invoked manually.
            run_early_update_stages(&mut c);

            // Users can construct their own context object by wrapping the `EngineContext`. Ideally
            // `ComfyGame` would be generic over this, but ... lifetimes are tough, and this is a
            // WIP. For now the solution is macros and/or copy pasting this trait impl.
            update(&mut GameContext {
                x: &mut state.x,
                y: &mut state.y,
                engine: &mut c,
            });

            // And here again
            run_late_update_stages(&mut c, delta());
        }
    }

    // Since we're passing `ComfyGame` to the game loop we need to provide
    // a way to get back the `EngineState`. Again just boilerplate sacrificed
    // to the holy crab.
    fn engine(&mut self) -> &mut EngineState {
        &mut self.engine
    }
}

// Keeping the same `setup` as other examples for clarity.
fn setup(state: &mut GameState, c: &mut EngineContext) {
    state.x = c.flags.borrow().len() as i32;
}

// The usual update, except we can now use only `GameContext`
fn update(c: &mut GameContext) {
    draw_text(
        &format!("I'm a very advanced example {} + {} = :O", c.x, c.y),
        Vec2::ZERO,
        WHITE,
        TextAlign::Center,
    );
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

pub async fn run() {
    let config = GameConfig {
        game_name: "Full Game Loop Example",
        // comfy includes a `define_versions!()` macro that creates a `version_str()`
        // function that returns a version from cargo & git.
        version: "v0.0.1",
        ..Default::default()
    };

    let engine = EngineState::new(config);
    let game = ComfyGame::new(engine);

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
