mod quicktype;

use comfy_core::*;
use grids::Grid;
use notify::{Config, RecommendedWatcher, Watcher};

pub use quicktype::*;
pub use serde_json;

pub fn parse_ldtk_map(
    map: &str,
) -> Result<quicktype::LdtkJson, serde_json::Error> {
    serde_json::from_str(map)
}

pub struct WorldMap {
    pub watcher: RecommendedWatcher,
    pub json: LdtkJson,
    pub recv: std::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    pub path: String,
}

impl WorldMap {
    pub fn new(json: LdtkJson, path: &str) -> Self {
        let (send, recv) = std::sync::mpsc::channel();

        let mut watcher =
            RecommendedWatcher::new(send, Config::default()).unwrap();
        watcher
            .watch(Path::new(path), notify::RecursiveMode::NonRecursive)
            .unwrap();

        Self { json, watcher, recv, path: path.to_string() }
    }

    pub fn maybe_reload(&mut self) {
        let mut reload_level = false;

        while let Ok(_event) = self.recv.try_recv() {
            reload_level = true;
        }

        if reload_level {
            match parse_ldtk_map(&std::fs::read_to_string(&self.path).unwrap())
            {
                // match deathmind_ldtk::serde_json::from_str(&std::fs::read_to_string(LDTK_PATH).unwrap()) {
                Ok(json) => {
                    println!("Reloaded map");
                    self.json = json;
                }
                Err(err) => {
                    println!("Error parsing map {err:?}");
                }
            }
        }
    }
}

pub trait LdtkLevelExtensions {
    fn id_position(&self, identifier: &str) -> Option<Vec2>;
}

impl LdtkLevelExtensions for Level {
    fn id_position(&self, identifier: &str) -> Option<Vec2> {
        let mut result = None;

        for layer in self.layer_instances.as_ref()?.iter() {
            layer.entity_instances.iter().for_each(|entity| {
                if entity.identifier == identifier {
                    let pos = vec2(
                        entity.px[0] as f32,
                        self.px_hei as f32 -
                            entity.px[1] as f32 -
                            layer.grid_size as f32,
                    );
                    result = Some(pos / layer.grid_size as f32);
                }
            });
        }

        result
    }
}

pub trait LdtkLayerExtensions {
    fn grid_to_world(&self, x: i32, y: i32) -> Vec2;
    fn px_to_world(&self, position: Vec2) -> Vec2;
}

impl LdtkLayerExtensions for LayerInstance {
    fn grid_to_world(&self, x: i32, y: i32) -> Vec2 {
        vec2(x as f32, self.c_hei as f32 - y as f32 - 1.0)
    }

    fn px_to_world(&self, position: Vec2) -> Vec2 {
        let grid = self.grid_size as f32;

        vec2(position.x / grid, self.c_hei as f32 - position.y / grid - 1.0)
    }
}

pub trait LdtkTileExtensions {
    fn to_world(&self, layer: &LayerInstance) -> Vec2;
}

impl LdtkTileExtensions for TileInstance {
    fn to_world(&self, layer: &LayerInstance) -> Vec2 {
        layer.px_to_world(vec2(self.px[0] as f32, self.px[1] as f32))
    }
}

pub trait LdtkEntityExtensions {
    fn bool_field(&self, name: &str) -> Option<bool>;
    fn str_field(&self, name: &str) -> Option<&str>;
    fn str_array_field(&self, name: &str) -> Option<Vec<String>>;
    fn entity_array_field(&self, name: &str) -> Option<Vec<String>>;
}

impl LdtkEntityExtensions for EntityInstance {
    fn bool_field(&self, name: &str) -> Option<bool> {
        self.field_instances
            .iter()
            .find(|x| x.identifier == name)
            .and_then(|x| x.value.as_ref())
            .and_then(|x| x.as_bool())
    }

    fn str_field(&self, name: &str) -> Option<&str> {
        self.field_instances
            .iter()
            .find(|x| x.identifier == name)
            .and_then(|x| x.value.as_ref())
            .and_then(|x| x.as_str())
    }

    fn str_array_field(&self, name: &str) -> Option<Vec<String>> {
        let field =
            self.field_instances.iter().find(|x| x.identifier == name)?;
        let array = field.value.as_ref()?.as_array()?;
        let strings = array.iter().filter_map(|x| x.as_str());
        Some(strings.map(|x| x.to_string()).collect_vec())
    }

    fn entity_array_field(&self, name: &str) -> Option<Vec<String>> {
        let field =
            self.field_instances.iter().find(|x| x.identifier == name)?;
        let array = field.value.as_ref()?.as_array()?;
        let strings = array
            .iter()
            .filter_map(|x| x.get("entityIid"))
            .filter_map(|x| x.as_str());
        Some(strings.map(|x| x.to_string()).collect_vec())
    }
}

pub fn grid_from_csv(layer: &LayerInstance) -> Grid<i32> {
    let width = layer.c_wid as i32;
    let height = layer.c_hei as i32;

    Grid::filled_with(width, height, |x, y| {
        layer.int_grid_csv[(x + y * width) as usize] as i32
    })
}
