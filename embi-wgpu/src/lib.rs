#![allow(clippy::uninlined_format_args)]
#![allow(clippy::new_without_default)]

pub use embi_core::*;
pub use wgpu::util::DeviceExt;

pub use winit::event::{
    ElementState, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode,
    WindowEvent,
};

mod blood_canvas;
mod bloom;
mod egui_integration;
mod game_loop;
mod instance;
mod pipelines;
mod post_processing;
mod render_pass;
mod renderer;
mod texture;
mod utils;

pub use crate::blood_canvas::*;
pub use crate::bloom::*;
pub use crate::egui_integration::*;
pub use crate::game_loop::*;
pub use crate::instance::*;
pub use crate::pipelines::*;
pub use crate::post_processing::*;
pub use crate::render_pass::*;
pub use crate::renderer::*;
pub use crate::texture::*;
pub use crate::utils::*;

pub use wgpu;
pub use wgpu_types;

pub trait RunGameLoop {
    fn one_frame(&mut self, delta: f32);
    fn title(&self) -> String;
    fn set_renderer(&mut self, renderer: WgpuRenderer);
    fn renderer(&mut self) -> &mut WgpuRenderer;
    fn resize(&mut self, new_size: UVec2);
    fn quit_flag(&mut self) -> bool;
}

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
    0 => Float32x3,
    1 => Float32x2,
    2 => Float32x4,
];

impl Vertex for SpriteVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>()
                as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

pub enum BufferType {
    Vertex,
    Index,
    Instance,
    Uniform,
    Storage,
}

impl BufferType {
    pub fn usage(&self) -> wgpu::BufferUsages {
        match self {
            BufferType::Vertex => {
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
            }
            BufferType::Index => {
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
            }
            BufferType::Instance => {
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
            }
            BufferType::Uniform => {
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
            BufferType::Storage => {
                todo!()
            }
        }
    }
}

pub struct UniformBindGroup {
    pub bind_group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
}

impl UniformBindGroup {
    pub fn simple(
        name: &str,
        device: &wgpu::Device,
        default_contents: &[u8],
    ) -> Self {
        let buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Uniform Buffer", name)),
                contents: default_contents,
                usage: wgpu::BufferUsages::UNIFORM |
                    wgpu::BufferUsages::COPY_DST,
            });

        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX |
                        wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        //
                        // has_dynamic_offset: true,
                        // min_binding_size: wgpu::BufferSize::new(
                        //     std::mem::size_of::<QuadUniform>() as u64,
                        // ),
                    },
                    count: None,
                }],
                label: Some(&format!("{} Bind Group Layout", name)),
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(&format!("{} Bind Group", name)),
        });

        Self { bind_group, layout, buffer }
    }
}

pub struct SizedBuffer {
    pub buffer: wgpu::Buffer,
    pub size: usize,
    pub buffer_type: BufferType,
    pub label: String,
}

impl SizedBuffer {
    pub fn new(
        label: &str,
        device: &wgpu::Device,
        size: usize,
        buffer_type: BufferType,
    ) -> Self {
        let desc = wgpu::BufferDescriptor {
            label: Some(label),
            usage: buffer_type.usage(),
            size: size as wgpu::BufferAddress,
            mapped_at_creation: false,
        };

        let buffer = device.create_buffer(&desc);

        Self { label: label.to_string(), size, buffer_type, buffer }
    }

    pub fn ensure_size_and_copy(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
    ) {
        if data.len() > self.size {
            self.buffer.destroy();
            self.size = data.len();
            self.buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&self.label),
                    usage: self.buffer_type.usage(),
                    contents: data,
                });
        } else {
            queue.write_buffer(&self.buffer, 0, data);
        }
    }
}

pub trait DeviceExtensions {
    fn simple_encoder(&self, label: &str) -> wgpu::CommandEncoder;
    fn simple_bind_group(
        &self,
        label: &str,
        texture: &Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup;
}

impl DeviceExtensions for wgpu::Device {
    fn simple_encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(label),
        })
    }

    fn simple_bind_group(
        &self,
        label: &str,
        texture: &Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        self.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        })
    }
}

pub trait CommandEncoderExtensions {
    fn simple_render_pass<'a>(
        &'a mut self,
        label: &str,
        clear_color: Option<Color>,
        view: &'a wgpu::TextureView,
        // depth_stencil_attachment: Option<
        //     wgpu::RenderPassDepthStencilAttachment,
        // >,
    ) -> wgpu::RenderPass;
}

impl CommandEncoderExtensions for wgpu::CommandEncoder {
    fn simple_render_pass<'a>(
        &'a mut self,
        label: &str,
        clear_color: Option<Color>,
        view: &'a wgpu::TextureView,
        // depth_stencil_attachment: Option<
        //     wgpu::RenderPassDepthStencilAttachment,
        // >,
    ) -> wgpu::RenderPass {
        self.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: color_to_clear_op(clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }
}

pub trait WgpuColorExtensions {
    fn to_wgpu(&self) -> wgpu::Color;
}

impl WgpuColorExtensions for Color {
    fn to_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

pub fn color_to_clear_op(color: Option<Color>) -> wgpu::LoadOp<wgpu::Color> {
    match color {
        Some(clear_color) => wgpu::LoadOp::Clear(clear_color.to_wgpu()),
        None => wgpu::LoadOp::Load,
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &MainCamera) {
        // Using Vec4 because of uniform 16 byte spacing requirement.
        self.view_position = camera.center.extend(0.0).extend(1.0).into();
        self.view_proj =
            camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

#[allow(dead_code)]
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);

pub fn create_render_pipeline(
    label: &str,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
    blend_mode: BlendMode,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    let blend_state = match blend_mode {
        BlendMode::Alpha => Some(wgpu::BlendState::ALPHA_BLENDING),
        // BlendMode::Additive => Some(wgpu::BlendState::ALPHA_BLENDING),
        // BlendMode::Additive => Some(wgpu::BlendState {
        //     color: wgpu::BlendComponent {
        //         src_factor: wgpu::BlendFactor::SrcAlpha,
        //         dst_factor: wgpu::BlendFactor::DstAlpha,
        //         operation: wgpu::BlendOperation::Add,
        //     },
        //     alpha: wgpu::BlendComponent {
        //         src_factor: wgpu::BlendFactor::One,
        //         dst_factor: wgpu::BlendFactor::One,
        //         operation: wgpu::BlendOperation::Add,
        //     }
        // }),
        BlendMode::Additive => {
            Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                // alpha: wgpu::BlendComponent::REPLACE,
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            })
        }
        BlendMode::None => Some(wgpu::BlendState::ALPHA_BLENDING),
    };

    // let blend_state = Some(wgpu::BlendState {
    //     color: wgpu::BlendComponent {
    //         src_factor: wgpu::BlendFactor::One,
    //         dst_factor: wgpu::BlendFactor::One,
    //         operation: wgpu::BlendOperation::Add,
    //     },
    //     // alpha: wgpu::BlendComponent::REPLACE,
    //     alpha: wgpu::BlendComponent {
    //         src_factor: wgpu::BlendFactor::One,
    //         dst_factor: wgpu::BlendFactor::One,
    //         operation: wgpu::BlendOperation::Add,
    //     },
    // });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: blend_state,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),

        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            // cull_mode: Some(wgpu::Face::Back),
            cull_mode: None,
            // Settings this to anything other than Fill requires
            // Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },

        depth_stencil: depth_format.map(|format| {
            wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }
        }),

        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn create_render_pipeline_with_layout(
    name: &str,
    device: &wgpu::Device,
    color_format: wgpu::TextureFormat,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
    blend_mode: BlendMode,
    enable_z_buffer: bool,
) -> wgpu::RenderPipeline {
    let layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Pipeline Layout", name)),
            bind_group_layouts,
            push_constant_ranges: &[],
        });


    create_render_pipeline(
        &format!("{} Pipeline", name),
        device,
        &layout,
        color_format,
        if enable_z_buffer { Some(Texture::DEPTH_FORMAT) } else { None },
        vertex_layouts,
        shader,
        blend_mode,
    )
}
