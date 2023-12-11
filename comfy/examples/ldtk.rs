use comfy::*;

simple_game!("LDTK Example", GameState, setup, update);

const LDTK_PATH: &str = "assets/comfy_ldtk.ldtk";

pub struct GameState {
    pub ldtk_map: LdtkWorldMap,
}

impl GameState {
    pub fn new(_c: &EngineState) -> Self {
        Self {
            ldtk_map: LdtkWorldMap::new(
                parse_ldtk_map(include_str!("../../assets/comfy_ldtk.ldtk"))
                    .unwrap(),
                LDTK_PATH,
            ),
        }
    }
}

fn setup(_state: &mut GameState, c: &mut EngineContext) {
    c.load_texture_from_bytes(
        "comfy",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/comfy.png"
        )),
    );

    c.load_texture_from_bytes(
        "tileset",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/tiles.png"
        )),
    );
}

fn update(state: &mut GameState, _c: &mut EngineContext) {
    clear_background(AQUAMARINE);

    let map = &state.ldtk_map.json;
    let level = &map.levels[0];
    for (i, layer) in
        level.layer_instances.as_ref().unwrap().iter().rev().enumerate()
    {
        let grid_size = layer.grid_size as f32;

        let tileset = layer
            .tileset_def_uid
            .and_then(|uid| map.defs.tilesets.iter().find(|t| t.uid == uid));

        if let Some(_tileset) = tileset {
            let texture = texture_id("tileset");

            // WORLD - TILES
            for tile in layer.grid_tiles.iter() {
                let pos = tile.to_world(layer);

                draw_sprite_ex(
                    texture,
                    pos,
                    WHITE,
                    10 + i as i32,
                    DrawTextureParams {
                        dest_size: Some(
                            splat(grid_size / 16.0).as_world_size(),
                        ),
                        source_rect: Some(IRect::new(
                            ivec2(tile.src[0] as i32, tile.src[1] as i32),
                            ivec2(grid_size as i32, grid_size as i32),
                        )),
                        flip_x: tile.f == 1 || tile.f == 3,
                        flip_y: tile.f == 2 || tile.f == 3,
                        ..Default::default()
                    },
                );
            }
        }

        // CHARACTERS - ENTITIES
        for entity in layer.entity_instances.iter() {
            if entity.identifier == "Character" {
                let center = entity.world_pos(layer.c_hei, layer.grid_size);
                let size = entity.world_size(layer.grid_size);

                draw_rect(center, size, RED.alpha(0.5), 100);
                draw_text(
                    entity.str_field("Name").unwrap_or_default(),
                    center,
                    BLACK,
                    TextAlign::Center,
                );
            } else {
                error!("Missing: entity {:?}", entity);
            }
        }
    }
}
