pub use embi::*;

define_versions!();

fn main() {
    color_backtrace::install();
    pollster::block_on(run());
}

pub async fn run() {
    cfg_if! {
        if #[cfg(feature = "demo")] {
             let game_name = "BITGUN Survivors (DEMO)";
        } else {
             let game_name = "BITGUN Survivors";
        }
    };

    let config = GameConfig {
        game_name,
        version: version_str(),

        hide_version: true,
        hide_menu_window: true,
        hide_upgrades: true,
        hide_title: true,
        hide_gallery: true,
        hide_controls: true,

        lighting: GlobalLightingParams {
            ambient_light_intensity: 1.0,

            ..Default::default()
        },

        ..Default::default()
    };


    let mob_animations = |name: &str| {
        hashmap! {
            ANIM_IDLE.to_string() => animation_frames(name, ivec2(0, 0), 1),
            ANIM_WALK.to_string() => animation_frames(name, ivec2(0, 1), 4),
            ANIM_ATTACK.to_string() => animation_frames(name, ivec2(0, 2), 4),
            ANIM_DEATH.to_string() => animation_frames(name, ivec2(0, 4), 1),
        }
    };

    let sprite_sheet = "atlas";

    fn mob_animations_ex(name: &str, anims: SimpleAnimations) -> Animations {
        let idle = anims.idle;
        let walk = anims.walk;
        let attack = anims.attack;
        let death = anims.death;

        hashmap! {
            ANIM_IDLE.to_string() => animation_frames(name, ivec2(0, idle.0), idle.1),
            ANIM_WALK.to_string() => animation_frames(name, ivec2(0, walk.0), walk.1),
            ANIM_ATTACK.to_string() => animation_frames(name, ivec2(0, attack.0), attack.1),
            ANIM_DEATH.to_string() => animation_frames(name, ivec2(0, death.0), death.1),
        }
    }

    let mob_config = hashmap! {
        ZOMBIE_BASIC => MobConfig {
            health: 30.0,
            shields: 0.0,
            size: 1.0,
            animations: mob_animations("zombie-basic"),
            speed: 1.0,
        },

        ZOMBIE_BOOMER => {
            MobConfig {
                health: 100.0,
                shields: 0.0,
                size: 2.0,
                animations: mob_animations_ex("zombie-boomer", SimpleAnimations {
                    idle: (0, 1),
                    walk: (0, 2),
                    attack: (2, 2),
                    death: (1, 1),
                }),
                speed: 0.7,
            }
        },

        ZOMBIE_SPIDER => {
            MobConfig {
                health: 45.0,
                shields: 0.0,
                size: 1.0,
                animations: mob_animations_ex("zombie-spider", SimpleAnimations {
                    idle: (0, 3),
                    walk: (1, 3),
                    attack: (2, 3),
                    death: (3, 1)
                }),
                speed: 1.5,
            }
        },

        ZOMBIE_BLOODED => {
            MobConfig {
                health: 70.0,
                shields: 40.0,
                size: 1.0,
                animations: mob_animations("zombie-blooded"),
                speed: 0.7,
            }
        },

        ZOMBIE_SPIKED => {
            MobConfig {
                health: 50.0,
                shields: 10.0,
                size: 1.0,
                animations: mob_animations("zombie-spiked"),
                speed: 1.2,
            }
        }
    };

    let player_sprite =
        Sprite::new(sprite_sheet.to_string(), splat(1.0), Z_PLAYER, WHITE)
            .with_rect(7 * 16, 2 * 16, 16, 16);

    let survivors_config =
        Arc::new(make_survivors_config(player_sprite, mob_config));
    let survivors_config_inner = survivors_config.clone();

    let game_state = Box::new(
        EngineState::new(
            config,
            Box::new(move |_c| {
                Arc::new(Mutex::new(SurvivorsGame::new(
                    survivors_config_inner.clone(),
                    |_c| {},
                    |_c| {},
                )))
            }),
        ), // .with_main_menu_update(Box::new(main_menu_update)),
    );

    crate::assets::load_assets();

    set_main_camera_zoom(30.0);

    run_qute_main_async(game_state).await;
}
