use comfy::*;

simple_game!("Cooldowns example", setup, update);

fn setup(c: &mut EngineContext) {
    c.load_sound_from_bytes(
        // Every sound gets a string name later used to reference it.
        "comfy-bell",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/bell-sfx.ogg"
        )),
        StaticSoundSettings::default(),
    );
}

struct SmallGlobalState {
    pub color: Color,
    pub revert_timer: f32,
}

// We could use a regular `GameState` as shown in the other examples,
// but lets also explore other _safe_ options for storing state.
//
// Many think that globals are a terrible thing. Comfy disagrees.
//
// Sometimes all you want is just a bit of state, and you don't want to
// restructure everything. Especially if the problem is nicely self-contained,
// and there are no obvious reasons for not doing so.
//
// If rust is good at one thing, it is refactoring, and even if you later
// decide you don't want a global state, you can safely move it elsewhere.
//
// That being said, _don't be ashamed to focus on your game first_, even if
// it means having a few global varaibles :)
static STATE: Lazy<AtomicRefCell<SmallGlobalState>> = Lazy::new(|| {
    AtomicRefCell::new(SmallGlobalState { color: WHITE, revert_timer: 0.0 })
});

fn update(_c: &mut EngineContext) {
    let mut state = STATE.borrow_mut();

    const COOLDOWN_TIME: f32 = 1.0;

    if is_key_pressed(KeyCode::Space) {
        // Comfy provides very easy immediate mode cooldowns that can be accessed
        // anywhere. All you need is a "key" and cooldown "time".
        //
        // When `.can_use()` is called the first time, it'll return true and start
        // the cooldown, and keep returning `false` until the engine ticks away at
        // the cooldown, at which point it will again be able to return `true`.
        //
        // If this feels complicated, play around with it! It will become a second
        // nature in no time.
        //
        // Note that mainly useful for things which have a "name". If you have 1000
        // enemies, you'd want to store their cooldowns together with their state,
        // as opposed to doing something like `format!("enemy-{}", id)`. But ... don't
        // be afraid to try this as well :)
        if cooldowns().can_use("bell", COOLDOWN_TIME) {
            state.color = RED;
            state.revert_timer = COOLDOWN_TIME;

            play_sound("comfy-bell");
        }
    }

    if state.revert_timer <= 0.0 {
        state.color = WHITE;
    } else {
        state.revert_timer -= delta();
    }

    draw_text(
        "Press SPACE to play SFX, but only once per second",
        Vec2::ZERO,
        state.color,
        TextAlign::Center,
    );
}
