use comfy::*;

// Unlike in many other examples, we'll need a custom GameState
// to know if our music player is playing.
simple_game!("Music Example", GameState, setup, update);

// GameState can be any struct that can store any fields, only
// requirement is that `GameState::new(c: &EngineContext)` exists.
struct GameState {
    pub music_playing: bool,
}


impl GameState {
    // We could use `EngineContext` to do additional engine setup,
    // or to initialize our state based on the engine context.
    //
    // For this example we don't need to use it, and just initialize
    // to a default value.
    pub fn new(_c: &mut EngineContext) -> Self {
        Self { music_playing: false }
    }
}

// When we pass `GameState` to `simple_game` we now also have to accept
// it as a parameter in `setup` and `update`.
fn setup(_state: &mut GameState, c: &mut EngineContext) {
    c.load_sound_from_bytes(
        "comfy-music",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/comfy-music.ogg"
        )),
        StaticSoundSettings::new().loop_region(..),
    );
}

fn update(state: &mut GameState, c: &mut EngineContext) {
    let (color, action) =
        if state.music_playing { (RED, "stop") } else { (WHITE, "play") };

    draw_text(
        &format!("Press Space to {} music", action),
        vec2(0.0, 1.0),
        color,
        TextAlign::Center,
    );

    if is_key_pressed(KeyCode::Space) {
        if state.music_playing {
            stop_sound("comfy-music");
        } else {
            play_sound("comfy-music");
        }

        state.music_playing = !state.music_playing;
    }

    draw_text(
        &format!("Press W/S to adjust master volume: {:.2}", master_volume()),
        vec2(0.0, 0.0),
        WHITE,
        TextAlign::Center,
    );

    if is_key_down(KeyCode::W) {
        change_master_volume(1.0 * c.delta as f64);
    }

    if is_key_down(KeyCode::S) {
        change_master_volume(-1.0 * c.delta as f64);
    }
}
