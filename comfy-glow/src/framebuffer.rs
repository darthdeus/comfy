use crate::prelude::*;

pub struct FrameBuffer {
    pub name: String,
    pub fbo: glow::NativeFramebuffer,
    pub rbo: glow::NativeRenderbuffer,
    pub color_buffer: glow::NativeTexture,
    gl: Arc<glow::Context>,
}

const INTERNAL_FORMAT: i32 = glow::RGBA16F as i32;
const FORMAT: u32 = glow::RGBA;
const TEXTURE_TYPE: u32 = glow::UNSIGNED_BYTE;

impl FrameBuffer {
    pub fn new(name: &str, resolution: IVec2, gl: Arc<glow::Context>) -> Self {
        unsafe {
            let (fbo, rbo, color_buffer) = Self::create(name, &gl, resolution);

            Self { name: name.to_string(), gl, fbo, rbo, color_buffer }
        }
    }

    pub unsafe fn resize(&mut self, gl: &glow::Context, resolution: IVec2) {
        self.gl.delete_framebuffer(self.fbo);
        self.gl.delete_renderbuffer(self.rbo);
        self.gl.delete_texture(self.color_buffer);

        let (fbo, rbo, color_buffer) = Self::create(&self.name, gl, resolution);

        self.fbo = fbo;
        self.rbo = rbo;
        self.color_buffer = color_buffer;
    }

    pub unsafe fn create(
        name: &str,
        gl: &glow::Context,
        resolution: IVec2,
    ) -> (glow::NativeFramebuffer, glow::NativeRenderbuffer, glow::NativeTexture)
    {
        let fbo =
            gl.create_framebuffer().expect("failed to create a framebuffer");

        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

        let color_buffer = gl.create_texture().unwrap();

        gl.safe_label(
            glow::FRAMEBUFFER,
            fbo.0.into(),
            Some(format!("FBO {}", name)),
        );

        {
            gl.bind_texture(glow::TEXTURE_2D, Some(color_buffer));
            gl.safe_label(
                glow::TEXTURE,
                color_buffer.0.into(),
                Some(format!("Texture {}", name)),
            );

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                INTERNAL_FORMAT,
                resolution.x,
                resolution.y,
                0,
                FORMAT,
                TEXTURE_TYPE,
                None,
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            gl.bind_texture(glow::TEXTURE_2D, None);

            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(color_buffer),
                0,
            );
        }

        let rbo =
            gl.create_renderbuffer().expect("failed to create renderbuffer");

        gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
        gl.renderbuffer_storage(
            glow::RENDERBUFFER,
            glow::DEPTH24_STENCIL8,
            resolution.x,
            resolution.y,
        );

        gl.framebuffer_renderbuffer(
            glow::FRAMEBUFFER,
            glow::DEPTH_STENCIL_ATTACHMENT,
            glow::RENDERBUFFER,
            Some(rbo),
        );

        let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);

        if status == glow::FRAMEBUFFER_COMPLETE {
            info!("Framebuffer {name}");
        } else {
            error!("Framebuffer {name} failed: {status}");
        }

        gl.bind_framebuffer(glow::FRAMEBUFFER, None);

        (fbo, rbo, color_buffer)
    }

    pub unsafe fn bind(&self) {
        self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.fbo);
            self.gl.delete_renderbuffer(self.rbo);
            self.gl.delete_texture(self.color_buffer);
        }
    }
}

pub struct TwoOutputFrameBuffer {
    pub fbo: glow::NativeFramebuffer,
    pub color_buffers: Vec<glow::NativeTexture>,
    gl: Arc<glow::Context>,
}

impl TwoOutputFrameBuffer {
    pub fn new(resolution: IVec2, gl: Arc<glow::Context>) -> Self {
        unsafe {
            let fbo = gl
                .create_framebuffer()
                .expect("failed to create a framebuffer");

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

            let mut color_buffers = vec![];

            for i in 0..2 {
                let color_buffer = gl.create_texture().unwrap();

                gl.bind_texture(glow::TEXTURE_2D, Some(color_buffer));

                gl.safe_label(
                    glow::TEXTURE,
                    color_buffer.0.into(),
                    Some(format!("first-pass-{}", i)),
                );

                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    INTERNAL_FORMAT,
                    resolution.x,
                    resolution.y,
                    0,
                    glow::RGB,
                    TEXTURE_TYPE,
                    None,
                );

                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::LINEAR as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::LINEAR as i32,
                );

                gl.bind_texture(glow::TEXTURE_2D, None);

                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0 + i,
                    glow::TEXTURE_2D,
                    Some(color_buffer),
                    0,
                );

                color_buffers.push(color_buffer);
            }

            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);

            if status == glow::FRAMEBUFFER_COMPLETE {
                info!("TwoOutputFrameBuffer created");
            } else {
                error!("TwoOutputFrameBuffer failed: {status}");
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            Self { fbo, gl, color_buffers }
        }
    }

    pub unsafe fn bind(&self) {
        self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
        self.gl
            .draw_buffers(&[glow::COLOR_ATTACHMENT0, glow::COLOR_ATTACHMENT1]);
    }
}

impl Drop for TwoOutputFrameBuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.fbo);
            for color_buffer in self.color_buffers.iter() {
                self.gl.delete_texture(*color_buffer);
            }
        }
    }
}
