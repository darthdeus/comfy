use crate::prelude::*;

#[derive(Debug)]
pub struct ReloadableString {
    pub data: String,
    pub path: String,
}

#[macro_export]
macro_rules! reloadable_str {
    ($path:expr) => {
        ReloadableString {
            data: include_str!($path).to_string(),
            path: $path.to_string(),
        }
    };
}

pub struct Shader {
    name: String,

    gl: Arc<glow::Context>,
    pub program: glow::Program,

    vertex: ReloadableString,
    fragment: ReloadableString,
}

impl Shader {
    pub fn new(
        name: &str,
        gl: Arc<glow::Context>,
        vertex: ReloadableString,
        fragment: ReloadableString,
    ) -> Self {
        info!("Compiling shader {name}");

        let program =
            Self::try_compile_shader(name, &gl, &vertex.data, &fragment.data)
                .expect("Initial shader compilation failed");

        Self {
            name: name.to_string(),
            gl,
            program,
            vertex,
            fragment,
        }
    }

    pub fn hot_reload(&mut self) {
        info!(
            "hot reloading {:?} {:?}",
            self.vertex.path, self.fragment.path
        );

        let result = (|| -> Result<glow::Program> {
            let prefix = std::path::Path::new("engine/src/shaders");

            let vp = prefix.join(
                &std::path::Path::new(&self.vertex.path).file_name().unwrap(),
            );
            let fp = prefix.join(
                &std::path::Path::new(&self.fragment.path)
                    .file_name()
                    .unwrap(),
            );

            let vp = std::fs::canonicalize(vp)?;
            let fp = std::fs::canonicalize(fp)?;

            let vertex = std::fs::read_to_string(&vp)?;
            let fragment = std::fs::read_to_string(&fp)?;

            Ok(Self::try_compile_shader(
                &self.name, &self.gl, &vertex, &fragment,
            )?)
        })();

        match result {
            Ok(program) => {
                info!("recompiled shader ok!");
                unsafe {
                    self.gl.delete_program(self.program);
                }
                self.program = program;
            }
            Err(err) => {
                error!(
                    "Failed to compile {} or {} ... err:\n{}",
                    self.vertex.path, self.fragment.path, err
                );
            }
        }
    }

    pub fn try_compile_shader(
        name: &str,
        gl: &Arc<glow::Context>,
        vertex: &str,
        fragment: &str,
    ) -> Result<glow::Program> {
        let shader_sources = [
            (glow::VERTEX_SHADER, vertex),
            (glow::FRAGMENT_SHADER, fragment),
        ];

        let shader_version = "#version 330 core";

        let program = unsafe {
            let mut shaders = Vec::with_capacity(shader_sources.len());

            let program = gl.create_program().expect("Cannot create program");

            for (i, (shader_type, shader_source)) in
                shader_sources.iter().enumerate()
            {
                let shader = gl
                    .create_shader(*shader_type)
                    .expect("Cannot create shader");

                gl.shader_source(
                    shader,
                    &format!("{}\n{}", shader_version, shader_source),
                );

                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    bail!("{}", gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);

                let type_str = match i {
                    0 => "vertex",
                    1 => "fragment",
                    _ => "UNKNOWN",
                };

                gl.safe_label(
                    glow::SHADER,
                    shader.0.into(),
                    Some(format!("shader {} {}", type_str, name)),
                );

                shaders.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                bail!("{}", gl.get_program_info_log(program));
            }

            gl.safe_label(
                glow::PROGRAM,
                program.0.into(),
                Some(format!("program {}", name)),
            );

            gl.use_program(Some(program));

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            program
        };

        Ok(program)
    }

    pub fn use_shader(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
        }
    }

    pub fn use_global(&self, name: &str) {
        self.set_float(name, GlobalParams::get(name))
    }

    pub fn use_global_int(&self, name: &str) {
        self.set_int(name, GlobalParams::get_int(name))
    }

    #[allow(dead_code)]
    pub fn set_bool(&self, name: &str, value: bool) {
        unsafe {
            self.gl.uniform_1_i32(
                self.gl.get_uniform_location(self.program, name).as_ref(),
                if value { 1 } else { 0 },
            );
        }
    }

    #[allow(dead_code)]
    pub fn set_mat4(&self, name: &str, mat: Mat4) {
        unsafe {
            self.gl.uniform_matrix_4_f32_slice(
                self.gl.get_uniform_location(self.program, name).as_ref(),
                false,
                mat.as_ref(),
            );
        }
    }

    #[allow(dead_code)]
    pub fn set_float(&self, name: &str, value: f32) {
        unsafe {
            self.gl.uniform_1_f32(
                self.gl.get_uniform_location(self.program, name).as_ref(),
                value,
            );
        }
    }

    #[allow(dead_code)]
    pub fn set_int(&self, name: &str, value: i32) {
        unsafe {
            self.gl.uniform_1_i32(
                self.gl.get_uniform_location(self.program, name).as_ref(),
                value,
            );
        }
    }

    #[allow(dead_code)]
    pub fn set_float_2(&self, name: &str, x: f32, y: f32) {
        unsafe {
            self.gl.uniform_2_f32(
                self.gl.get_uniform_location(self.program, name).as_ref(),
                x,
                y,
            );
        }
    }

    #[allow(dead_code)]
    pub fn set_float_3(&self, name: &str, x: f32, y: f32, z: f32) {
        unsafe {
            self.gl.uniform_3_f32(
                self.gl.get_uniform_location(self.program, name).as_ref(),
                x,
                y,
                z,
            );
        }
    }

    #[allow(dead_code)]
    pub fn set_float_4(&self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        unsafe {
            self.gl.uniform_4_f32(
                self.gl.get_uniform_location(self.program, name).as_ref(),
                x,
                y,
                z,
                w,
            );
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
        }
    }
}
