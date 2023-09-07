use crate::*;

pub struct Instance {
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
    pub color: Vec4,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Mat4::from_translation(self.position) *
                Mat4::from_rotation_z(self.rotation) *
                Mat4::from_scale(self.scale.extend(1.0)))
            .to_cols_array_2d(),

            color: self.color.into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

impl InstanceRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4,
    ];
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance.
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance.
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
            // attributes: &[
            //     wgpu::VertexAttribute {
            //         offset: 0,
            //         // Vertex shader currently only uses 0 and 1, but will
            // use more soon.         shader_location: 2,
            //         format: wgpu::VertexFormat::Float32x4,
            //     },
            //     // A mat4 takes up 4 vertex slots as it is technically 4
            // vec4s. We need     // to define a slot for each vec4.
            // We'll have to reassemble the mat4     // in the
            // shader.     wgpu::VertexAttribute {
            //         offset: mem::size_of::<[f32; 4]>() as
            // wgpu::BufferAddress,         shader_location: 3,
            //         format: wgpu::VertexFormat::Float32x4,
            //     },
            //     wgpu::VertexAttribute {
            //         offset: mem::size_of::<[f32; 8]>() as
            // wgpu::BufferAddress,         shader_location: 4,
            //         format: wgpu::VertexFormat::Float32x4,
            //     },
            //     wgpu::VertexAttribute {
            //         offset: mem::size_of::<[f32; 12]>() as
            // wgpu::BufferAddress,         shader_location: 5,
            //         format: wgpu::VertexFormat::Float32x4,
            //     },
            //     wgpu::VertexAttribute {
            //         offset: mem::size_of::<[f32; 16]>() as
            // wgpu::BufferAddress,         shader_location: 6,
            //         format: wgpu::VertexFormat::Float32x3,
            //     },
            // ],
        }
    }
}
