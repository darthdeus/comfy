use blobs::*;
use comfy::*;

// This example shows an integration between comfy and blobs, a simple 2d physics engine. It's not
// the most beautiful example, and maybe a bit verbose for what it does, but it tries to showcase
// some more extensible ways of using comfy.
comfy_game!(
    "Physics Example",
    GameContext,
    GameState,
    make_context,
    setup,
    update
);

pub enum BallSpawningSpeed {
    Comfy,
    Uncomfy,
}

pub struct GameState {
    pub spawn_timer: f32,
    pub physics: Physics,
    pub ball_spawning_speed: BallSpawningSpeed,
}

impl GameState {
    pub fn new(_c: &mut EngineContext) -> Self {
        Self {
            spawn_timer: 0.0,
            physics: Physics::new(vec2(0.0, -20.0), false),
            ball_spawning_speed: BallSpawningSpeed::Comfy,
        }
    }
}

pub struct GameContext<'a, 'b: 'a> {
    // While we could access delta through .engine, it's easier to just expose it once and then
    // benefit all over the codebase.
    pub delta: f32,
    // We'll continually spawn balls using a simple f32 timer.
    pub spawn_timer: &'a mut f32,
    pub ball_spawning_speed: &'a mut BallSpawningSpeed,
    pub physics: &'a mut Physics,
    pub engine: &'a mut EngineContext<'b>,
}

fn make_context<'a, 'b: 'a>(
    state: &'a mut GameState,
    engine: &'a mut EngineContext<'b>,
) -> GameContext<'a, 'b> {
    GameContext {
        spawn_timer: &mut state.spawn_timer,
        delta: engine.delta,
        ball_spawning_speed: &mut state.ball_spawning_speed,
        physics: &mut state.physics,
        engine,
    }
}

fn setup(c: &mut GameContext) {
    let rbd_handle = c.physics.insert_rbd(RigidBodyBuilder::new().build());

    c.physics.insert_collider_with_parent(
        ColliderBuilder::new().build(),
        rbd_handle,
    );

    // We can add a circle constraint which prevents any body from leaving the constrained area.
    // Note that currently blobs constraints work based on position and ignore collider radius.
    c.physics
        .constraints
        .push(Constraint { position: Vec2::ZERO, radius: 5.0 });

    // We'll need SFX for this
    c.engine.load_sound_from_bytes(
        // Every sound gets a string name later used to reference it.
        "comfy-bell",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/bell-sfx.ogg"
        )),
        StaticSoundSettings::default(),
    );
}

fn update(c: &mut GameContext) {
    *c.spawn_timer -= c.delta;

    let limit = match c.ball_spawning_speed {
        BallSpawningSpeed::Comfy => 100,
        BallSpawningSpeed::Uncomfy => 1000,
    };

    if *c.spawn_timer <= 0.0 && c.physics.rbd_count() < limit {
        let max_time = match c.ball_spawning_speed {
            BallSpawningSpeed::Comfy => 0.4,
            BallSpawningSpeed::Uncomfy => 0.01,
        };

        // Every time the timer expires we reset it to a random value.
        *c.spawn_timer = random_range(0.01, max_time);

        let rbd_handle = c.physics.insert_rbd(
            RigidBodyBuilder::new().position(random_circle(2.0)).build(),
        );

        c.physics.insert_collider_with_parent(
            ColliderBuilder::new().radius(random_range(0.05, 0.4)).build(),
            rbd_handle,
        );

        play_sound("comfy-bell");
    }

    c.physics.step(c.delta as f64);

    // We could iterate rigid bodies individually, but blobs also has a nice way of collecting
    // debug data in a simple format for rendering/logginc that we'll use to draw all bodies and
    // colliders.
    let debug = c.physics.debug_data();

    for body in debug.bodies.iter() {
        draw_circle(body.transform.translation, 0.1, RED.alpha(0.8), 5);
    }

    for constraint in c.physics.constraints.iter() {
        draw_circle(
            constraint.position,
            constraint.radius,
            WHITE.alpha(0.1),
            3,
        );
    }

    // Let's use the colliders arena to draw the colliders for a change. We could
    // also just use `debug.colliders`.
    for (_, collider) in c.physics.col_set.iter() {
        // We'll draw collider circles behind the rigid body circles using a lower z_index.
        draw_circle(
            collider.absolute_transform.translation,
            collider.radius,
            BLUE.alpha(0.5),
            4,
        );
    }

    draw_text(
        "Be warned, blobs isn't very stable with many balls :)",
        Position::screen_percent(0.5, 0.85).to_world(),
        WHITE,
        TextAlign::Center,
    );

    draw_text(
        &format!("There are now {} balls", c.physics.rbd_count()),
        Position::screen_percent(0.5, 0.15).to_world(),
        WHITE,
        TextAlign::Center,
    );

    egui::Window::new("More balls")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
        .show(egui(), |ui| {
            match c.ball_spawning_speed {
                BallSpawningSpeed::Comfy => {
                    if ui.button("Make it go faster").clicked() {
                        *c.ball_spawning_speed = BallSpawningSpeed::Uncomfy;
                    }
                }
                BallSpawningSpeed::Uncomfy => {
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("NOOOO, PLS STOP!!!").size(
                                rescale(
                                    c.physics.rbd_count() as f32,
                                    0.0..limit as f32,
                                    10.0..100.0,
                                ),
                            ),
                        ))
                        .clicked()
                    {
                        *c.ball_spawning_speed = BallSpawningSpeed::Comfy;
                        c.physics.reset();
                    }
                }
            }
        });
}
