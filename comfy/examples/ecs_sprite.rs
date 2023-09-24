use comfy::*;

simple_game!("ECS Sprite Example", setup, update);

struct Player;

fn setup(c: &mut EngineContext) {
    c.load_texture_from_bytes(
        // Every texture gets a string name later used to reference it.
        "comfy",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/comfy.png"
        )),
    );

    // Spawn a new entity with a sprite and a transform.
    c.commands().spawn((
        Sprite::new("comfy".to_string(), vec2(1.0, 1.0), 100, WHITE),
        Transform::position(vec2(0.0, 0.0)),
        // Use simple struct to tag your entities to be able to query them later.
        Player,
    ));
}

fn update(c: &mut EngineContext) {
    // Query all entities with Player component, Sprite component and Transform component.
    for (_, (_, _, transform)) in
        c.world().query::<(&Player, &Sprite, &mut Transform)>().iter()
    {
        transform.scale = (get_time() as f32).sin() + 2.0;
    }
}
