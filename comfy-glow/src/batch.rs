use crate::*;

pub struct StaticBatch {
    pub vao: glow::NativeVertexArray,
    pub vbo: glow::NativeBuffer,
    pub ebo: glow::NativeBuffer,

    pub shader: Shader,

    pub gl: Arc<glow::Context>,

    pub textures: Vec<Texture>,
    // pub character_atlas: Texture,
    // pub background: Texture,
    // pub tiles: Texture,
}

impl StaticBatch {
    pub unsafe fn new(
        // asset_loader: &mut AssetLoader,
        gl: Arc<glow::Context>,
        shader: Shader,
    ) -> Self {
        let vbo = gl.create_buffer().expect("failed to create vbo");
        let ebo = gl.create_buffer().expect("failed to create ebo");

        let vao = gl.create_vertex_array().expect("Cannot create vertex array");

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

        gl.safe_label(glow::VERTEX_ARRAY, vao.0.into(), Some("Batch VAO"));
        gl.safe_label(glow::BUFFER, vbo.0.into(), Some("Batch VBO"));
        gl.safe_label(glow::BUFFER, ebo.0.into(), Some("Batch EBO"));

        // gl.attrib_f32(0, 3, 9, 0);
        // gl.attrib_f32(1, 4, 9, 3);
        // gl.attrib_f32(2, 2, 9, 7);

        gl.attrib_f32(0, 3, 9, 0);
        gl.attrib_f32(1, 2, 9, 3);
        gl.attrib_f32(2, 4, 9, 5);

        gl.bind_vertex_array(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

        // let t1 =
        //     Texture::new(gl.clone(), include_bytes!("../assets/wall.jpg"));

        // let character_atlas = asset_loader.texture("atlas.png");
        // let background = asset_loader.texture("pozadi.png");
        // let tiles = asset_loader.texture("tilemaps/atlas.png");

        Self {
            shader,
            vbo,
            vao,
            ebo,

            gl: gl.clone(),

            textures: vec![],
            // character_atlas,
            // background,
            // tiles,
        }
    }

    pub unsafe fn bind(&self) {
        self.gl.bind_vertex_array(Some(self.vao));
        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
        self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
    }

    pub unsafe fn upload(&self, vertices: &[f32], indices: &[u32]) {
        self.gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(vertices),
            glow::STATIC_DRAW,
        );

        self.gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(indices),
            glow::STATIC_DRAW,
        );
    }

    pub unsafe fn prepare(&self, vertices: &[f32], indices: &[u32]) {
        self.bind();
        self.upload(vertices, indices);
    }
}

impl Drop for StaticBatch {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_buffer(self.ebo);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}
