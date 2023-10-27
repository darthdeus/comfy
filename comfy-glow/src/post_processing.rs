use crate::prelude::*;

pub struct PostProcessingSettings {
    pub bloom: bool,
}

pub trait PostProcessingStep {
    unsafe fn bind_begin(&self);
    unsafe fn bind_end(&self);
    unsafe fn draw(&self, params: &PostProcessingParams);
}

#[derive(Copy, Clone, Debug)]
pub struct PostProcessingParams<'a> {
    // pub config: &'a Config,
    // pub input: &'a Input,
    pub flags: &'a HashMap<String, bool>,
    pub frame: u32,
    pub delta: f32,
    pub time: f32,
    pub resolution: UVec2,
}

pub struct PostProcessing {
    name: String,

    shader: Shader,
    fb: FrameBuffer,

    vbo: glow::NativeBuffer,
    vao: glow::NativeVertexArray,

    gl: Arc<glow::Context>,
}

impl PostProcessing {
    pub fn new(
        name: &str,
        shader: Shader,
        resolution: IVec2,
        gl: &Arc<glow::Context>,
    ) -> Self {
        unsafe {
            #[rustfmt::skip]
            let quad_vertices: Vec<f32> = vec![
                // positions   // texCoords
                -1.0,  1.0,   0.0, 1.0,
                -1.0, -1.0,   0.0, 0.0,
                 1.0, -1.0,   1.0, 0.0,

                -1.0,  1.0,   0.0, 1.0,
                 1.0, -1.0,   1.0, 0.0,
                 1.0,  1.0,   1.0, 1.0
            ];

            // #[rustfmt::skip]
            // let quad_vertices: Vec<f32> = vec![
            //     // positions   // texCoords
            //     -1.0,  1.0, 0.0,  0.0, 1.0,
            //     -1.0, -1.0, 0.0,  0.0, 0.0,
            //      1.0, -1.0, 0.0,  1.0, 0.0,
            //
            //     -1.0,  1.0, 0.0,  0.0, 1.0,
            //      1.0, -1.0, 0.0,  1.0, 0.0,
            //      1.0,  1.0, 0.0,  1.0, 1.0
            // ];

            let vao = gl.create_vertex_array().expect("failed to create VAO");
            let vbo = gl.create_buffer().expect("failed to create VBO");

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            gl.safe_label(
                glow::VERTEX_ARRAY,
                vao.0.into(),
                Some(format!("VAO {}", name)),
            );
            gl.safe_label(
                glow::BUFFER,
                vbo.0.into(),
                Some(format!("VBO {}", name)),
            );

            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&quad_vertices),
                glow::STATIC_DRAW,
            );


            // gl.enable_vertex_attrib_array(0);
            // gl.attrib_f32(0, 3, 5, 0);
            // gl.enable_vertex_attrib_array(1);
            // gl.attrib_f32(1, 3, 5, 3);

            gl.enable_vertex_attrib_array(0);
            gl.attrib_f32(0, 2, 4, 0);
            gl.enable_vertex_attrib_array(1);
            gl.attrib_f32(1, 2, 4, 2);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Self {
                name: name.to_string(),

                shader,
                fb: FrameBuffer::new(name, resolution, gl.clone()),
                vbo,
                vao,
                gl: gl.clone(),
            }
        }
    }

    pub fn resize(&mut self, new_resolution: IVec2) {
        self.fb = FrameBuffer::new(&self.name, new_resolution, self.gl.clone());
    }

    pub unsafe fn bind_begin(&self) {
        self.fb.bind();
        // self.gl.viewport(0, 0, 800, 600);
    }

    pub unsafe fn bind_end(&self) {
        self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
    }

    pub unsafe fn draw(&self, params: &PostProcessingParams) {
        self.draw_from(params, self.fb.color_buffer);
    }

    pub unsafe fn draw_from(
        &self,
        params: &PostProcessingParams,
        source_texture: glow::NativeTexture,
    ) {
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(glow::COLOR_BUFFER_BIT);

        self.gl.disable(glow::DEPTH_TEST);

        self.shader.use_shader();

        let offset = 1.0 / 300.0;

        #[rustfmt::skip]
        let offsets = [
            -offset, offset,  // top-left
            0.0, offset,      // top-center
            offset, offset,   // top-right
            -offset, 0.0,     // center-left
            0.0, 0.0,         // center-center
            offset, 0.0,      // center - right
            -offset, -offset, // bottom-left
            0.0, -offset,     // bottom-center
            offset, -offset,  // bottom-right
        ];

        self.gl.uniform_2_f32_slice(
            self.gl
                .get_uniform_location(self.shader.program, "offsets")
                .as_ref(),
            &offsets,
        );

        self.gl.bind_texture(glow::TEXTURE_2D, Some(source_texture));

        // gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
        // TODO: !!!!
        // TODO: !!!!
        // TODO: !!!!
        // TODO: !!!!
        // TODO: !!!!
        // self.shader.set_float_3(
        //     "iResolution",
        //     p.config.physical_resolution.x,
        //     p.config.physical_resolution.y,
        //     1.0,
        // );
        //
        // self.shader.set_float_4(
        //     "iMouse",
        //     // mouse_world.x,
        //     // mouse_world.y,
        //     p.input.mouse_position.to_raw().x,
        //     p.config.physical_resolution.y - p.input.mouse_position.to_raw().y,
        //     0.0,
        //     0.0,
        // );

        self.shader.use_global("contrast");
        self.shader.use_global("brightness");
        self.shader.use_global("saturation");
        self.shader.use_global("gamma");
        self.shader.use_global("chromatic_aberration");

        self.shader.set_float("iTime", params.time);
        self.shader.set_float("iTimeDelta", params.delta);
        self.shader.set_int("iFrame", params.frame as i32);
        self.shader.set_float("iFrameRate", 1.0 / params.delta);
        self.shader.set_float_2(
            "iResolution",
            params.resolution.x as f32,
            params.resolution.y as f32,
        );

        self.shader.set_int("iChannel0", 0);
        self.shader.set_int("iChannel1", 1);
        self.shader.set_int("iChannel2", 2);
        self.shader.set_int("iChannel3", 3);

        self.shader.set_bool(
            "skip_pp",
            *params.flags.get("skip_pp").unwrap_or(&false),
        );

        // self.shader
        //     .set_bool("shake", *params.flags.get("shake").unwrap_or(&false));
        self.shader.set_bool("shake", true);
        self.shader
            .set_float("shake_amount", GlobalParams::get("shake_amount"));

        #[rustfmt::skip]
        let edge_kernel = [
            -1, -1, -1,
            -1,  8, -1,
            -1, -1, -1
        ];

        self.gl.uniform_1_i32_slice(
            self.gl
                .get_uniform_location(self.shader.program, "edge_kernel")
                .as_ref(),
            &edge_kernel,
        );

        #[rustfmt::skip]
        let blur_kernel = [
            1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0,
            2.0 / 16.0, 4.0 / 16.0, 2.0 / 16.0,
            1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0
        ];

        self.gl.uniform_1_f32_slice(
            self.gl
                .get_uniform_location(self.shader.program, "blur_kernel")
                .as_ref(),
            &blur_kernel,
        );

        immediate_draw_quad(&self.gl);

        // self.gl.bind_vertex_array(Some(self.vao));
        // self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
        // self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
    }

    pub fn hot_reload(&mut self) {
        self.shader.hot_reload();
    }
}

impl Drop for PostProcessing {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

impl PostProcessingStep for PostProcessing {
    unsafe fn draw(&self, params: &PostProcessingParams) {
        self.draw(params);
    }

    unsafe fn bind_begin(&self) {
        self.bind_begin();
    }

    unsafe fn bind_end(&self) {
        self.bind_end();
    }
}
