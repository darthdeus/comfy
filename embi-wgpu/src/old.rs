pub fn render_sprites(
    &mut self,
    clear_color: Color,
    draw_queue: Vec<SpriteDraw>,
) {
    let _span = span!("render_sprites");
    let textures = self.textures.borrow();

    let mut encoder = self.device.simple_encoder("Sprite Render Encoder");

    {
        let mut render_pass = encoder.simple_render_pass(
            "Sprite Render Pass",
            Some(clear_color),
            &self.palette_texture.view,
        );

        // draw_queue.sort_by_key(|(texture, _, _, z_index, _)| texture.id);
        // draw_queue.sort_by_key(|(texture, _, _, z_index, _)| *z_index);

        let sorted_instances = draw_queue
            .into_iter()
            .map(|draw| {
                (draw.texture, Instance {
                    // position: position.to_world().extend(0.0),
                    position: draw.position.extend(0.0),
                    color: draw.color.to_vec4(),
                    rotation: draw.raw_draw.rotation,
                    // scale: Vec2::ONE,
                    scale: draw
                        .raw_draw
                        .dest_size
                        .unwrap_or(Vec2::new(1.0, 1.0)), /* .unwrap_or(Size::world(1.0, 1.0))
                                                          * .to_world(), */
                })
            })
            .collect_vec();

        let instance_data = sorted_instances
            .iter()
            .map(|x| Instance::to_raw(&x.1))
            .collect_vec();

        let data = bytemuck::cast_slice(instance_data.as_slice());

        self.sprite_instance_buffer.ensure_size_and_copy(
            &self.device,
            &self.queue,
            data,
        );

        render_pass.set_pipeline(&self.sprite_render_pipeline);
        render_pass.set_vertex_buffer(0, self.sprite_vertex_buffer.slice(..));
        render_pass
            .set_vertex_buffer(1, self.sprite_instance_buffer.buffer.slice(..));

        render_pass.set_index_buffer(
            self.sprite_index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        if sorted_instances.len() > 0 {
            let mut range_start = 0;
            let mut current_texture = sorted_instances[0].0;

            let mut draw_range = |tex: &TextureHandle, a: u32, b: u32| {
                let bind_group =
                    textures.get(tex).map(|x| &x.0).unwrap_or_else(|| {
                        &textures.get(&texture_path("error")).unwrap().0
                    });
                // .unwrap_or(&self.error_bind_group);

                render_pass.set_bind_group(0, bind_group, &[]);

                render_pass.draw_indexed(
                    0..QUAD_INDICES_U16.len() as u32,
                    0,
                    a..b as _,
                );
            };

            for (i, (texture, _)) in sorted_instances.iter().enumerate() {
                if texture != &current_texture {
                    draw_range(&current_texture, range_start, i as u32);

                    range_start = i as u32;
                    current_texture = *texture;
                }
            }

            draw_range(
                &current_texture,
                range_start,
                sorted_instances.len() as u32,
            );
        }
    }

    self.queue.submit(std::iter::once(encoder.finish()));
}
