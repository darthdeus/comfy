use comfy::*;

simple_game!("Version", update);

fn update(_c: &mut EngineContext) {
    draw_text(
        &format!("The version of this game is: {}", version_str()),
        vec2(0.0, 1.0),
        WHITE,
        TextAlign::Center,
    );

    draw_text(
        "Comfy allows embedding the current crate version (and optionally git \
         commit) with `version_str()`\nTo embed the current git commit \
         compile with `--feature comfy/git-version`",
        vec2(0.0, -1.0),
        WHITE,
        TextAlign::Center,
    );
}
