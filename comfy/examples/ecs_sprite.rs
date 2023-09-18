use comfy::*;

example_game!("ECS Sprite Example", setup, update);

struct Player;

fn setup(c: &mut EngineContext) {
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
    for (entity, (player, sprite, transform)) in
        c.world().query::<(&Player, &Sprite, &mut Transform)>().iter()
    {
        transform.scale += 10.0 * c.delta;
    }
}
