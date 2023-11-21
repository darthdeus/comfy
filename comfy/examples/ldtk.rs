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
                parse_ldtk_map(include_str!(
                    "../../assets/comfy_ldtk.ldtk"
                ))
                .unwrap(),
                LDTK_PATH,
            )
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
    for (i, layer) in level.layer_instances.as_ref().unwrap().iter().rev().enumerate()
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






// pub fn draw_ldtk_map(ldtk: &LdtkWorldMap, spatial: &mut SpatialHash) {
//     let map = &ldtk.json;
//     let level = &map.levels[0];

//     for (i, layer) in
//         level.layer_instances.as_ref().unwrap().iter().rev().enumerate()
//     {
//         let grid_size = layer.grid_size as f32;

//         if layer.layer_instance_type == "IntGrid" {
//             let grid = grid_from_csv(layer);

//             for y in 0..grid.height {
//                 for x in 0..grid.width {
//                     if grid[(x, y)] == 1 {
//                         let pos = layer.grid_to_world(x, y);
//                         spatial.add_shape(
//                             AabbShape::shape(pos, splat(1.0)),
//                             UserData::default(),
//                         );
//                     }
//                 }
//             }
//         }

//         let tileset = layer
//             .tileset_def_uid
//             .and_then(|uid| map.defs.tilesets.iter().find(|t| t.uid == uid));

//         if let Some(_tileset) = tileset {
//             // let path = map.resolve_rel_level_path(tileset.rel_path.as_ref().unwrap());
//             // let texture = c.asset_loader.texture(&path);
//             let texture = texture_id("camp");

//             for tile in layer.grid_tiles.iter() {
//                 // let pos = vec2(
//                 //     tile.px[0] as f32,
//                 //     //tile.px[1] as f32,
//                 //     (layer.c_hei as f32 - 1.0) * grid_size - tile.px[1] as f32,
//                 // );

//                 let pos = tile.to_world(layer);

//                 // draw_text_ex(
//                 //     &format!("{:.0?}", pos),
//                 //     pos.as_world(),
//                 //     TextAlign::Center,
//                 //     TextParams {
//                 //         font: egui::FontId::new(
//                 //             8.0 / egui_scale_factor(),
//                 //             egui::FontFamily::Proportional,
//                 //         ),
//                 //         color: RED,
//                 //         ..Default::default()
//                 //     },
//                 // );

//                 draw_sprite_ex(
//                     texture,
//                     pos,
//                     // pos / 16.0,
//                     // WHITE.alpha(0.51),
//                     WHITE,
//                     10 + i as i32,
//                     DrawTextureParams {
//                         dest_size: Some(
//                             splat(grid_size / 16.0).as_world_size(),
//                         ),
//                         source_rect: Some(IRect::new(
//                             ivec2(tile.src[0] as i32, tile.src[1] as i32),
//                             ivec2(grid_size as i32, grid_size as i32),
//                         )),
//                         flip_x: tile.f == 1 || tile.f == 3,
//                         flip_y: tile.f == 2 || tile.f == 3,
//                         ..Default::default()
//                     },
//                 );
//             }
//         }

//         for entity in layer.entity_instances.iter() {
//             if entity.identifier == "Zone" {
//                 let center = entity.world_pos(layer.c_hei, layer.grid_size);
//                 let size = entity.world_size(layer.grid_size);

//                 draw_rect(center, size, RED.alpha(0.5), 100);
//             } else {
//                 error!("TODO: entity {:?}", entity);
//             }
//             // state.objects.create_or_update(
//             //     entity,
//             //     layer.c_hei as f32,
//             //     grid_size,
//             // );
//         }
//     }
// }
