use crate::prelude::*;

pub struct Bloom {
    gl: Arc<glow::Context>,
    pingpong: [FrameBuffer; 2],

    threshold_shader: Shader,
    blur_shader: Shader,
    pub blend_shader: Shader,
}

impl Bloom {
    pub fn new(resolution: IVec2, gl: Arc<glow::Context>) -> Self {
        let pingpong = [
            FrameBuffer::new("bloom-1", resolution, gl.clone()),
            FrameBuffer::new("bloom-2", resolution, gl.clone()),
        ];

        let threshold_shader = Shader::new(
            "bloom-threshold",
            gl.clone(),
            reloadable_str!("shaders/simple.vert"),
            reloadable_str!("shaders/bloom-threshold.frag"),
        );

        let blur_shader = Shader::new(
            "bloom-blur",
            gl.clone(),
            reloadable_str!("shaders/simple.vert"),
            reloadable_str!("shaders/bloom-blur.frag"),
        );

        let blend_shader = Shader::new(
            "bloom-blend",
            gl.clone(),
            reloadable_str!("shaders/simple.vert"),
            reloadable_str!("shaders/bloom-blend.frag"),
        );

        info!("BLOOM created");

        Self { gl, pingpong, threshold_shader, blur_shader, blend_shader }
    }

    pub unsafe fn resize(&mut self, resolution: IVec2) {
        self.pingpong[0].resize(&self.gl, resolution);
        self.pingpong[1].resize(&self.gl, resolution);
    }

    pub fn result_texture(&self) -> glow::NativeTexture {
        self.pingpong[0].color_buffer
    }

    pub unsafe fn draw_threshold(
        &mut self,
        source_texture: glow::NativeTexture,
    ) {
        self.pingpong[0].bind();
        self.threshold_shader.use_shader();
        self.threshold_shader.use_global("bloomThreshold");
        self.threshold_shader.use_global("colorScale");

        self.gl.bind_texture(glow::TEXTURE_2D, Some(source_texture));
        draw_quad(&self.gl);
        self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
    }

    pub unsafe fn draw_blur(&mut self) {
        let mut horizontal = true;
        let mut first_iteration = true;

        let amount = 20;

        if GlobalParams::get_int("clear_bloom") == 1 {
            for i in 0..2 {
                self.pingpong[i].bind();
                self.gl.disable(glow::DEPTH_TEST);
                self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
                self.gl.clear(glow::COLOR_BUFFER_BIT);
            }
        }

        self.blur_shader.use_shader();

        for _ in 0..amount {
            let i = if horizontal { 1 } else { 0 };
            self.blur_shader.set_bool("horizontal", horizontal);
            self.pingpong[i].bind();

            let tex = if first_iteration {
                self.pingpong[0].color_buffer
            } else {
                self.pingpong[if horizontal { 0 } else { 1 }].color_buffer
            };

            self.gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            draw_quad(&self.gl);

            horizontal = !horizontal;

            if first_iteration {
                first_iteration = false;
            }
        }

        self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
    }

    pub unsafe fn draw_blend(&mut self, source_tex: glow::NativeTexture) {
        self.gl.disable(glow::DEPTH_TEST);
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(glow::COLOR_BUFFER_BIT);

        self.gl.active_texture(glow::TEXTURE0);
        self.gl.bind_texture(glow::TEXTURE_2D, Some(source_tex));
        self.gl.active_texture(glow::TEXTURE1);
        self.gl.bind_texture(glow::TEXTURE_2D, Some(self.result_texture()));
        self.gl.active_texture(glow::TEXTURE0);

        self.blend_shader.use_shader();
        self.blend_shader.set_int("scene", 0);
        self.blend_shader.set_int("bloomBlur", 1);
        self.blend_shader.use_global("exposure");
        // TODO: remove this in favor of late gamma?
        self.blend_shader.use_global("bloomGamma");
        self.blend_shader.use_global("colorScale");
        self.blend_shader.use_global_int("tonemapping_alg");

        draw_quad(&self.gl);
    }
}
