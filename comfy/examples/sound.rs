use comfy::*;

simple_game!("Sound Example", setup, update);

fn setup(_c: &mut EngineContext) {
    load_sound_from_bytes(
        // Every sound gets a string name later used to reference it.
        "comfy-bell",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/bell-sfx.ogg"
        )),
        StaticSoundSettings::default(),
    );
}

fn update(_c: &mut EngineContext) {
    let color = if is_key_down(KeyCode::Space) { RED } else { WHITE };

    draw_text("Press SPACE to play SFX", Vec2::ZERO, color, TextAlign::Center);

    if is_key_pressed(KeyCode::Space) {
        play_sound("comfy-bell");
    }
}
