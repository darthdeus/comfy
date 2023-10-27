pub use sdl2::{event::Event, video::Window};

pub use comfy_core::*;

pub use glow;
pub use glow::HasContext;

pub use sdl2;

pub use crate::batch::*;
pub use crate::blood_canvas::*;
pub use crate::bloom::*;
pub use crate::egui_integration::*;
pub use crate::framebuffer::*;
pub use crate::post_processing::*;
pub use crate::reloadable_str;
pub use crate::renderer::*;
pub use crate::shader::*;
pub use crate::texture::*;
pub use crate::*;

pub trait GlExtensions {
    unsafe fn attrib_f32(
        &self,
        index: i32,
        size: i32,
        stride: i32,
        offset: i32,
    );

    unsafe fn set_wrap_mode(&self, wrap_mode: WrapMode);

    unsafe fn safe_label<S>(
        &self,
        identifier: u32,
        name: u32,
        label: Option<S>,
    ) where
        S: AsRef<str>;

    unsafe fn push_safe_group<S>(&self, source: u32, id: u32, message: S)
    where S: AsRef<str>;

    unsafe fn pop_safe_group(&self);
}

impl GlExtensions for glow::Context {
    unsafe fn attrib_f32(
        &self,
        index: i32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        let f32_size = std::mem::size_of::<f32>() as i32;

        self.vertex_attrib_pointer_f32(
            index as u32,
            size,
            glow::FLOAT,
            false,
            stride * f32_size,
            offset * f32_size,
        );

        self.enable_vertex_attrib_array(index as u32);
    }

    unsafe fn set_wrap_mode(&self, wrap_mode: WrapMode) {
        let wrap_mode = match wrap_mode {
            WrapMode::Clamp => glow::CLAMP_TO_EDGE,
            WrapMode::Repeat => glow::REPEAT,
            WrapMode::MirroredRepeat => glow::MIRRORED_REPEAT,
        } as i32;

        self.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            wrap_mode,
        );

        self.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            wrap_mode,
        );
    }

    unsafe fn safe_label<S>(
        &self,
        identifier: u32,
        name: u32,
        label: Option<S>,
    ) where
        S: AsRef<str>,
    {
        #[cfg(not(target_os = "macos"))]
        self.object_label(identifier, name, label);
    }

    unsafe fn push_safe_group<S>(&self, source: u32, id: u32, message: S)
    where S: AsRef<str> {
        #[cfg(not(target_os = "macos"))]
        self.push_debug_group(source, id, message);
    }

    unsafe fn pop_safe_group(&self) {
        #[cfg(not(target_os = "macos"))]
        self.pop_debug_group();
    }
}
