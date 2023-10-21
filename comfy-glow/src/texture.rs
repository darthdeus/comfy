use crate::*;

#[derive(Clone, Debug)]
pub struct AtlasSprite {
    pub texture: Texture,
    pub rows: i32,
    pub cols: i32,
    pub index: i32,
}

#[derive(Clone, Debug)]
pub struct Texture {
    gl: Arc<glow::Context>,
    // TODO glow::Texture vs glow::NativeTexture
    pub texture: glow::Texture,
    pub width: u32,
    pub height: u32,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture);
        }
    }
}

pub enum WrapMode {
    Clamp,
    #[allow(dead_code)]
    Repeat,
    #[allow(dead_code)]
    MirroredRepeat,
}

impl Texture {
    pub fn new(name: &str, gl: Arc<glow::Context>, bytes: &[u8]) -> Self {
        let image = image::load_from_memory(bytes).unwrap();
        let width = image.width();
        let height = image.height();
        let tex_data = image.flipv();

        Self::from_rgba_bytes(name, gl, &tex_data.to_rgba8(), width, height)
    }

    pub fn handle(&self) -> TextureHandle {
        TextureHandle::Raw(self.texture.0.get() as u64)
    }

    // TODO; duplicate code
    pub fn from_image(
        name: &str,
        gl: Arc<glow::Context>,
        image: &DynamicImage,
    ) -> Self {
        let width = image.width();
        let height = image.height();
        let tex_data = image.flipv();

        Self::from_rgba_bytes(name, gl, &tex_data.to_rgba8(), width, height)
    }

    pub fn from_rgba_bytes(
        name: &str,
        gl: Arc<glow::Context>,
        bytes: &[u8],
        width: u32,
        height: u32,
    ) -> Texture {
        info!("Creating texture {name}");

        unsafe {
            let texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.safe_label(
                glow::TEXTURE,
                texture.0.into(),
                Some(format!("texture {name}")),
            );

            gl.set_wrap_mode(WrapMode::Clamp);

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as _,
            );
            // gl.tex_parameter_i32(
            //     glow::TEXTURE_2D,
            //     glow::TEXTURE_MIN_FILTER,
            //     glow::NEAREST_MIPMAP_NEAREST as _,
            // );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as _,
            );

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as _,
                width as _,
                height as _,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&bytes),
            );

            // gl.generate_mipmap(glow::TEXTURE_2D);

            gl.bind_texture(glow::TEXTURE_2D, None);

            Self { gl: gl.clone(), texture, width, height }
        }
    }

    pub fn bind(&self, gl: &glow::Context, shader: &Shader, index: usize) {
        unsafe {
            let unit = glow::TEXTURE0 + index as u32;

            gl.active_texture(unit);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            shader.set_int(&format!("texture{}", index), index as i32);
        }
    }

    // TODO
    // pub fn atlas_sprite(
    //     &self,
    //     rows: u32,
    //     cols: u32,
    //     x: u32,
    //     y: u32,
    // ) -> DrawTextureParams {
    //     let w = self.width / cols;
    //     let h = self.height / rows;
    //
    //     let x_off = (x % cols) * w;
    //     let y_off = (y % rows) * h;
    //
    //     DrawTextureParams {
    //         dest_size: DestSize::Fixed(Vec2::new(w as f32, h as f32)),
    //         source: Some(Rect::from_top_left_u32(x_off, y_off, w, h)),
    //         ..Default::default()
    //     }
    // }
}

#[test]
pub fn test_idx() {
    assert_eq!(glow::TEXTURE1, glow::TEXTURE0 + 1);
    assert_eq!(glow::TEXTURE2, glow::TEXTURE0 + 2);
    assert_eq!(glow::TEXTURE3, glow::TEXTURE0 + 3);
    assert_eq!(glow::TEXTURE4, glow::TEXTURE0 + 4);
    assert_eq!(glow::TEXTURE5, glow::TEXTURE0 + 5);
    assert_eq!(glow::TEXTURE6, glow::TEXTURE0 + 6);
    assert_eq!(glow::TEXTURE7, glow::TEXTURE0 + 7);
    assert_eq!(glow::TEXTURE8, glow::TEXTURE0 + 8);
}
