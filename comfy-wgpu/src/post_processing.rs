use crate::*;

pub fn insert_post_processing_effect(
    renderer: &WgpuRenderer,
    index: i32,
    name: &str,
    shader: Shader,
) {
    let effect = PostProcessingEffect::new(
        name.to_string(),
        &renderer.context.device,
        &[
            &renderer.context.texture_layout,
            &renderer.global_lighting_params_bind_group_layout,
        ],
        &renderer.config,
        renderer.render_texture_format,
        shader.clone(),
        &mut renderer.shaders.borrow_mut(),
    );

    let mut effects = renderer.post_processing_effects.borrow_mut();

    if index == -1 {
        effects.push(effect);
    } else if index >= 0 {
        effects.insert(index as usize, effect);
    } else {
        panic!("Invalid index = {}, must be -1 or non-negative.", index);
    }

    renderer.shaders.borrow_mut().insert(shader.id, shader);
}

pub struct PostProcessingEffect {
    pub id: ShaderId,
    pub name: String,
    pub enabled: bool,
    pub render_texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
}

impl PostProcessingEffect {
    pub fn new_with_mip(
        name: String,
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        config: &wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
        shader: Shader,
        shaders: &mut ShaderMap,
        mip_level_count: u32,
        blend: wgpu::BlendState,
    ) -> Self {
        let render_texture = Texture::create_scaled_mip_surface_texture(
            device,
            config,
            format,
            1.0,
            mip_level_count,
            &name,
        );

        let bind_group = device.simple_bind_group(
            &format!("{} Post Processing Bind Group", name),
            &render_texture,
            bind_group_layouts[0],
        );

        let id = shader.id;

        let pipeline = create_post_processing_pipeline(
            &name,
            device,
            format,
            bind_group_layouts,
            shader.clone(),
            blend,
        );

        shaders.insert(shader.id, shader);

        Self { id, name, enabled: true, render_texture, bind_group, pipeline }
    }

    pub fn new(
        name: String,
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        config: &wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
        shader: Shader,
        shaders: &mut ShaderMap,
    ) -> Self {
        Self::new_with_mip(
            name,
            device,
            bind_group_layouts,
            config,
            format,
            shader,
            shaders,
            1,
            wgpu::BlendState::REPLACE,
        )
    }
}

pub const USER_SHADER_PREFIX: &str = concat!(
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/structs.wgsl")),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/user_post_processing_vertex.wgsl"
    ))
);

pub fn create_user_shader_module(
    device: &wgpu::Device,
    shader: &Shader,
) -> wgpu::ShaderModule {
    let full_shader = format!("{}{}", USER_SHADER_PREFIX, &shader.source);

    let descriptor = wgpu::ShaderModuleDescriptor {
        label: Some(&shader.name),
        source: wgpu::ShaderSource::Wgsl(full_shader.as_str().into()),
    };

    device.create_shader_module(descriptor)
}

pub fn create_post_processing_pipeline(
    name: &str,
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    shader: Shader,
    blend: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    // let shader = create_user_shader_module(device, &shader);
    let shader = device.create_shader_module(shader.to_wgpu());

    let pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Post Processing Pipeline Layout", name)),
            bind_group_layouts,
            push_constant_ranges: &[],
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

    render_pass.push_debug_group(name);

    render_pass.set_pipeline(post_processing_pipeline);
    render_pass.set_bind_group(0, post_processing_bind_group, &[]);
    render_pass.set_bind_group(1, lighting_params_bind_group, &[]);

    if let Some(blend) = blend_constant {
        render_pass.set_blend_constant(wgpu::Color {
            r: blend,
            g: blend,
            b: blend,
            a: 1.0,
        });
    }

    render_pass.draw(0..3, 0..1);

    render_pass.pop_debug_group();
}
