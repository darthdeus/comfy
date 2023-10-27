mod batch;
mod blood_canvas;
mod bloom;
mod egui_integration;
mod framebuffer;
mod post_processing;
mod prelude;
mod renderer;
#[macro_use]
mod shader;
mod texture;

pub use crate::prelude::*;

static QUAD_BUFFERS: OnceCell<QuadBuffers> = OnceCell::new();

pub struct GlSafeGroup<'a> {
    gl: &'a glow::Context,
    #[cfg(feature = "tracy")]
    _span: Option<tracy_client::Span>,
}

impl Drop for GlSafeGroup<'_> {
    fn drop(&mut self) {
        unsafe {
            self.gl.pop_safe_group();
        }
    }
}

#[macro_export]
macro_rules! gl_span {
    ($gl: expr, $name: expr) => {{
        $gl.push_safe_group(glow::DEBUG_SOURCE_APPLICATION, 0, $name);

        GlSafeGroup {
            gl: &$gl,
            #[cfg(feature = "tracy")]
            _span: Some(tracy_client::span!($name, 0)),
        }
    }};
}

pub fn mouse_button_try_from_sdl(
    button: sdl2::mouse::MouseButton,
) -> Option<MouseButton> {
    match button {
        sdl2::mouse::MouseButton::Left => Some(MouseButton::Left),
        sdl2::mouse::MouseButton::Middle => Some(MouseButton::Middle),
        sdl2::mouse::MouseButton::Right => Some(MouseButton::Right),
        sdl2::mouse::MouseButton::Unknown => None,
        sdl2::mouse::MouseButton::X1 => None,
        sdl2::mouse::MouseButton::X2 => None,
    }
}

struct QuadBuffers {
    vao: glow::NativeVertexArray,
    vbo: glow::NativeBuffer,
}

pub unsafe fn immediate_draw_quad(gl: &glow::Context) {
    let buffers = QUAD_BUFFERS
        .get()
        .expect("init_quad_buffers must be called before immediate_draw_quad");

    gl.bind_vertex_array(Some(buffers.vao));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffers.vbo));
    // gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
    // gl.bind_vertex_array(None);
    gl.draw_arrays(glow::TRIANGLES, 0, 6);
    gl.bind_vertex_array(None);
}

pub unsafe fn init_quad_buffers(gl: Arc<glow::Context>) {
    // #[rustfmt::skip]
    // let quad_vertices = [
    //     // positions        // texture Coords
    //     // -1.0,  1.0, 0.0, 0.0, 1.0,
    //     // -1.0, -1.0, 0.0, 0.0, 0.0,
    //     //  1.0,  1.0, 0.0, 1.0, 1.0,
    //     //  1.0, -1.0, 0.0, 1.0, 0.0,
    //
    //     // positions   // texCoords
    //     -1.0,  1.0, 0.0,  0.0, 1.0,
    //     -1.0, -1.0, 0.0,  0.0, 0.0,
    //      1.0, -1.0, 0.0,  1.0, 0.0,
    //
    //     -1.0,  1.0, 0.0,  0.0, 1.0,
    //      1.0, -1.0, 0.0,  1.0, 0.0,
    //      1.0,  1.0, 0.0,  1.0, 1.0
    // ];

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

    let vao = gl.create_vertex_array().expect("QUAD VAO must succeed");
    let vbo = gl.create_buffer().expect("QUAD VBO must succeed");

    gl.bind_vertex_array(Some(vao));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        bytemuck::cast_slice(&quad_vertices),
        glow::STATIC_DRAW,
    );

    gl.safe_label(glow::VERTEX_ARRAY, vao.0.into(), Some("VAO QUAD"));
    gl.safe_label(glow::BUFFER, vbo.0.into(), Some("VBO QUAD"));

    // gl.enable_vertex_attrib_array(0);
    // gl.attrib_f32(0, 3, 5, 0);
    // gl.enable_vertex_attrib_array(1);
    // gl.attrib_f32(1, 2, 5, 3);

    gl.enable_vertex_attrib_array(0);
    gl.attrib_f32(0, 2, 4, 0);
    gl.enable_vertex_attrib_array(1);
    gl.attrib_f32(1, 2, 4, 2);

    gl.bind_vertex_array(None);

    let quad_buffers = QuadBuffers { vao, vbo };

    QUAD_BUFFERS.set(quad_buffers).unwrap_or_else(|_| {
        panic!("QUAD_BUFFERS must be initialized only once")
    });
}
