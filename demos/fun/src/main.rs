use comfy::*;
use gltf::Gltf;


comfy_game!("Something deeply sacrilegious", Maybe3D);

pub struct Maybe3D {
    monkey: tobj::Model,

    sponza: gltf::Gltf,
}

impl GameLoop for Maybe3D {
    fn new(_c: &mut EngineState) -> Self {
        let path = std::path::Path::new(
            "../Vulkan-Samples-Assets/scenes/sponza/Sponza01.gltf",
        );

        let path = std::env::current_dir().unwrap().join(path);

        println!("reading path {}", path.display());

        let gltf = Gltf::open(path).unwrap();

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                println!("{:#?}", node);
            }
        }

        let (mut model, materials) =
            tobj::load_obj("../suzanne.obj", &tobj::GPU_LOAD_OPTIONS).unwrap();

        let _materials = materials.unwrap();

        assert_eq!(model.len(), 1, "monkey had more than one monkey");

        Self { monkey: model.remove(0), sponza: gltf }
    }

    fn update(&mut self, _c: &mut EngineContext) {
        clear_background(BLACK);

        let mesh = &self.monkey.mesh;

        let vertices = mesh
            .positions
            .iter()
            .chunks(3)
            .into_iter()
            .map(|mut chunk| {
                let x = chunk.next().unwrap();
                let y = chunk.next().unwrap();
                let z = chunk.next().unwrap();

                SpriteVertex {
                    position: [*x, *y, *z],
                    color: WHITE.to_array_f32(),
                    tex_coords: Default::default(),
                }
            })
            .collect_vec();

        draw_mesh(Mesh {
            vertices: vertices.into(),
            indices: mesh.indices.clone().into(),
            ..Default::default()
        });

        let buffers = self.sponza.buffers().collect_vec();

        let blob = self.sponza.blob.as_ref();

        for scene in self.sponza.scenes() {
            for node in scene.nodes() {
                if let Some(mesh) = node.mesh() {
                    for primitive in mesh.primitives() {
                        // let reader = primitive.reader(|buffer| {
                        //     let buffer = &buffers[buffer.index()];
                        //     match buffer.source() {
                        //         gltf::buffer::Source::Bin => buffer.index(),
                        //         gltf::buffer::Source::Uri(uri) => {
                        //             panic!("uri not supported: {uri}");
                        //         }
                        //     }
                        // });
                    }
                }
            }
        }
    }
}
