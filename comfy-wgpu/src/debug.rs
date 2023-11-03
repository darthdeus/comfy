use crate::*;

pub fn render_debug(
    context: &GraphicsContext,
    enable_z_buffer: bool,
    quad_ubg: &UniformBindGroup,
    texture_layout: &wgpu::BindGroupLayout,
    depth_texture: &Arc<Texture>,
    bind_groups: Vec<&wgpu::BindGroup>,
    pipelines: &mut HashMap<String, wgpu::RenderPipeline>,
    surface_view: &wgpu::TextureView,
) {
    let _span = span!("render_debug");

    let size = 0.3;

    let quads: Vec<_> = bind_groups
        .iter()
        .enumerate()
        .map(|(i, _)| {
            QuadUniform {
                clip_position: [
                    1.0 - size / 2.0, // - size / 3.0 * i as f32,
                    0.8 - size / 2.0 - 2.0 * size * i as f32,
                ],
                size: [size, size],
            }
        })
        .collect();

    let debug_render_pipeline = pipelines
        .entry(if enable_z_buffer { "debug-z" } else { "debug" }.into())
        .or_insert_with(|| {
            create_render_pipeline_with_layout(
                "Debug",
                &context.device,
                context.config.borrow().format,
                &[&texture_layout, &quad_ubg.layout],
                &[],
                // TODO: .shaders.get_or_err(...)
                &reloadable_wgsl_shader!("debug"),
                BlendMode::Alpha,
                enable_z_buffer,
            )
        });


    for (i, bind_group) in bind_groups.iter().enumerate() {
        context.queue.write_buffer(
            &quad_ubg.buffer,
            0,
            bytemuck::cast_slice(&[quads[i]]),
        );

        let mut encoder = context.device.simple_encoder("Debug Render Encoder");
        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Debug Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: surface_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: color_to_clear_op(None),
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: depth_stencil_attachment(
                        enable_z_buffer,
                        &depth_texture.view,
                        false,
                    ),
                });


            render_pass.set_pipeline(debug_render_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_bind_group(1, &quad_ubg.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        context.queue.submit(std::iter::once(encoder.finish()));
    }
}
