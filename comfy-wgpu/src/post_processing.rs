use crate::*;

pub struct PostProcessingEffect {
    pub name: Cow<'static, str>,
    pub enabled: bool,
    pub render_texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
}

impl PostProcessingEffect {
    pub fn new_with_mip(
        name: &'static str,
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        config: &wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
        shader: wgpu::ShaderModuleDescriptor,
        mip_level_count: u32,
        blend: wgpu::BlendState,
    ) -> Self {
        let render_texture = Texture::create_scaled_mip_surface_texture(
            device,
            config,
            format,
            1.0,
            mip_level_count,
            name,
        );

        let bind_group = device.simple_bind_group(
            &format!("{} Post Processing Bind Group", name),
            &render_texture,
            bind_group_layouts[0],
        );

        let pipeline = create_post_processing_pipeline(
            name,
            device,
            format,
            bind_group_layouts,
            shader,
            blend,
        );

        Self {
            name: name.into(),
            enabled: true,
            render_texture,
            bind_group,
            pipeline,
        }
    }

    pub fn new(
        name: &'static str,
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        config: &wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
        shader: wgpu::ShaderModuleDescriptor,
    ) -> Self {
        Self::new_with_mip(
            name,
            device,
            bind_group_layouts,
            config,
            format,
            shader,
            1,
            wgpu::BlendState::REPLACE,
        )
    }
}

pub fn create_post_processing_pipeline(
    name: &str,
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    shader: wgpu::ShaderModuleDescriptor,
    blend: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    let pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Post Processing Pipeline Layout", name)),
            bind_group_layouts,
            #[cfg(not(feature = "push-constants"))]
            push_constant_ranges: &[],
            #[cfg(feature = "push-constants")]
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::FRAGMENT,
                range: 0..4,
            }],
        });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{} Post Processing Pipeline", name)),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(blend),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn draw_post_processing_output(
    name: &str,
    encoder: &mut wgpu::CommandEncoder,
    post_processing_pipeline: &wgpu::RenderPipeline,
    post_processing_bind_group: &wgpu::BindGroup,
    lighting_params_bind_group: &wgpu::BindGroup,
    target_view: &wgpu::TextureView,
    should_clear: bool,
    blend_constant: Option<f64>,
    push_constants: Option<&[u8]>,
) {
    let mut render_pass =
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("{} Post Processing Render Pass", name)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if should_clear {
                        wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        })
                    } else {
                        wgpu::LoadOp::Load
                    },
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

    render_pass.set_pipeline(post_processing_pipeline);
    render_pass.set_bind_group(0, post_processing_bind_group, &[]);
    render_pass.set_bind_group(1, lighting_params_bind_group, &[]);

    if let Some(push_constants) = push_constants {
        if cfg!(feature = "push-constants") {
            render_pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                0,
                push_constants,
            );
        } else {
            panic!(
                "push constants are temporarily disabled while wgpu fixes \
                 them."
            );
        }
    }

    if let Some(blend) = blend_constant {
        render_pass.set_blend_constant(wgpu::Color {
            r: blend,
            g: blend,
            b: blend,
            a: 1.0,
        });
    }

    render_pass.draw(0..3, 0..1);
}
