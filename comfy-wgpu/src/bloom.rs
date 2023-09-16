use crate::*;

// Blend factor & settings taken from bevy

pub const BLOOM_MIP_LEVEL_COUNT: u32 = 10;

const BLUR_DIR_ZERO: [u32; 4] = [0, 0, 0, 0];
const BLUR_DIR_ONE: [u32; 4] = [1, 0, 0, 0];


pub struct FrameBuffer {
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

impl FrameBuffer {
    pub fn new(
        name: &str,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let texture = Texture::create_scaled_mip_filter_surface_texture(
            device,
            config,
            format,
            1.0,
            1,
            wgpu::FilterMode::Linear,
            &format!("{} Texture", name),
        );

        let bind_group = device.simple_bind_group(
            &format!("{} Bind Group", name),
            &texture,
            layout,
        );

        Self { texture, bind_group }
    }
}

pub struct Bloom {
    pub context: GraphicsContext,

    pub format: wgpu::TextureFormat,
    pub threshold: PostProcessingEffect,
    pub mipmap_generator: MipmapGenerator,
    // pub blur_bind_group_layout: wgpu::BindGroupLayout,
    pub blur_texture: Texture,
    pub blur_bind_group: wgpu::BindGroup,
    pub mip_blur_pipeline: wgpu::RenderPipeline,

    pub merge_pipeline: wgpu::RenderPipeline,

    pub gaussian_pipeline: wgpu::RenderPipeline,

    pub blur_direction_buffer_0: wgpu::Buffer,
    pub blur_direction_buffer_1: wgpu::Buffer,
    pub blur_direction_group_0: wgpu::BindGroup,
    pub blur_direction_group_1: wgpu::BindGroup,
    pub blur_direction_layout: wgpu::BindGroupLayout,

    pub pingpong: [FrameBuffer; 2],

    pub lighting_params: Arc<wgpu::BindGroup>,
}

impl Bloom {
    pub fn new(
        context: &GraphicsContext,
        config: &wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
        lighting_params: Arc<wgpu::BindGroup>,
        lighting_params_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // let threshold = PostProcessingEffect::new_with_mip(
        //     "Bloom Threshold",
        //     device,
        //     &layout,
        //     config,
        //     format,
        //     simple_fragment_shader(
        //         "bloom-threshold",
        //         include_str!("../../assets/shaders/bloom-threshold.wgsl"),
        //     ),
        //     BLOOM_MIP_LEVEL_COUNT,
        //     wgpu::BlendState::REPLACE,
        // );

        let device = &context.device;
        let texture_layout = &context.texture_layout;

        let threshold_render_texture =
            Texture::create_scaled_mip_surface_texture(
                device,
                config,
                format,
                1.0,
                BLOOM_MIP_LEVEL_COUNT,
                "Bloom Threshold Texture",
            );

        let threshold = PostProcessingEffect {
            name: "Bloom Threshold".into(),
            enabled: true,
            bind_group: device.simple_bind_group(
                "Bloom Threshold Bind Group",
                &threshold_render_texture,
                texture_layout,
            ),
            render_texture: threshold_render_texture,
            pipeline: create_post_processing_pipeline(
                "Bloom Threshold",
                device,
                format,
                &[texture_layout, lighting_params_layout],
                reloadable_wgsl_fragment_shader!("bloom-threshold").into(),
                wgpu::BlendState::REPLACE,
            ),
        };

        // simple_fragment_shader(
        //     "bloom-threshold",
        //     include_str!("../../assets/shaders/bloom-threshold.wgsl"),
        // ),

        // let use_hdr = true;
        // let hdr_format = wgpu::TextureFormat::Rgba16Float;


        // let blur_bind_group_layout = device.simple_bind_group("Bloom Blur", texture, layout)
        // let blur_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         label: Some("Bloom Blur Bind Group Layout"),
        //         entries: &[
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 0,
        //                 visibility: wgpu::ShaderStages::FRAGMENT,
        //                 ty: wgpu::BindingType::Texture {
        //                     sample_type: wgpu::TextureSampleType::Float {
        //                         filterable: true, // !use_hdr,
        //                     },
        //                     view_dimension: wgpu::TextureViewDimension::D2,
        //                     multisampled: false,
        //                 },
        //                 count: None,
        //             },
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 1,
        //                 visibility: wgpu::ShaderStages::FRAGMENT,
        //                 ty: wgpu::BindingType::Sampler(
        //                     // if false && use_hdr {
        //                     //     wgpu::SamplerBindingType::NonFiltering
        //                     // } else {
        //                     wgpu::SamplerBindingType::Filtering, // },
        //                 ),
        //                 count: None,
        //             },
        //         ],
        //     });


        let blur_texture = Texture::create_scaled_mip_filter_surface_texture(
            device,
            config,
            format,
            1.0,
            1,
            wgpu::FilterMode::Linear,
            "Bloom Blur Texture",
        );

        let blur_bind_group = device.simple_bind_group(
            "Bloom Blur Bind Group",
            &blur_texture,
            texture_layout,
        );

        let mip_blur_pipeline = create_post_processing_pipeline(
            "Bloom Blur",
            device,
            format,
            &[texture_layout],
            simple_fragment_shader(
                "bloom-mip-blur",
                include_str!("../../assets/shaders/bloom-mip-blur.wgsl"),
            ),
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Constant,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::REPLACE,
            },
        );

        let merge_pipeline = create_post_processing_pipeline(
            "Bloom Merge",
            device,
            format,
            &[texture_layout],
            simple_fragment_shader(
                "bloom-mip-blur",
                include_str!("../../assets/shaders/bloom-merge.wgsl"),
            ),
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Constant,
                    dst_factor: wgpu::BlendFactor::OneMinusConstant,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::REPLACE,
            },
        );

        let mipmap_generator = MipmapGenerator::new(device, format);

        let pingpong = [
            FrameBuffer::new(
                "Bloom Ping Pong 0",
                device,
                config,
                format,
                texture_layout,
            ),
            FrameBuffer::new(
                "Bloom Ping Pong 1",
                device,
                config,
                format,
                texture_layout,
            ),
        ];

        let blur_direction_layout = context.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Bloom Blur Direction Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        );

        let blur_direction_buffer_0 = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Bloom Blur Direction Buffer = 0"),
                contents: bytemuck::cast_slice(&[BLUR_DIR_ZERO]),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );

        let blur_direction_buffer_1 = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Bloom Blur Direction Buffer = 1"),
                contents: bytemuck::cast_slice(&[BLUR_DIR_ONE]),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );

        let blur_direction_group_0 =
            context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bloom Blur Direction Bind Group = 0"),
                layout: &blur_direction_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: blur_direction_buffer_0.as_entire_binding(),
                }],
            });

        let blur_direction_group_1 =
            context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bloom Blur Direction Bind Group = 1"),
                layout: &blur_direction_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: blur_direction_buffer_1.as_entire_binding(),
                }],
            });


        let gaussian_pipeline = create_post_processing_pipeline(
            "Bloom Gaussian",
            device,
            format,
            &[texture_layout, &lighting_params_layout, &blur_direction_layout],
            simple_fragment_shader(
                "bloom-gauss",
                include_str!("../../assets/shaders/bloom-gauss.wgsl"),
            ),
            wgpu::BlendState::REPLACE,
        );


        Self {
            context: context.clone(),

            format,
            threshold,
            mipmap_generator,
            // mipmaps, mipmaps_bind_group,

            // blur_bind_group_layout,
            blur_texture,
            blur_bind_group,
            mip_blur_pipeline,

            blur_direction_buffer_0,
            blur_direction_buffer_1,
            blur_direction_group_0,
            blur_direction_group_1,

            blur_direction_layout,

            merge_pipeline,

            pingpong,
            gaussian_pipeline,

            lighting_params,
        }
    }

    pub fn draw(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        first_pass_bind_group: &wgpu::BindGroup,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        draw_post_processing_output(
            "Bloom Threshold",
            encoder,
            &self.threshold.pipeline,
            first_pass_bind_group,
            &self.lighting_params,
            &self.threshold.render_texture.view,
            true,
            None,
        );

        {
            let mut horizontal = true;
            let mut first_iteration = true;

            let amount = 20;

            for iter in 0..amount {
                let i = if horizontal { 1 } else { 0 };

                let tex = if first_iteration {
                    &self.threshold.bind_group
                } else {
                    &self.pingpong[if horizontal { 0 } else { 1 }].bind_group
                };

                let horizontal_u: u32 = i as u32;


                // draw_post_processing_output(
                //     encoder,
                //     &self.gaussian_pipeline,
                //     tex,
                //     &self.lighting_params,
                //     // &self.threshold.bind_group,
                //     &self.pingpong[i].texture.view,
                //     true,
                //     None,
                // );

                {
                    // self.context.queue.write_buffer(
                    //     &self.blur_direction_buffer,
                    //     0,
                    //     bytemuck::cast_slice(&[if horizontal { 0 } else { 1 }]),
                    // );

                    let mut render_pass = encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor {
                            label: Some(&format!(
                                "Bloom Pingpong {} Post Processing Render Pass",
                                iter
                            )),
                            color_attachments: &[Some(
                                wgpu::RenderPassColorAttachment {
                                    view: &self.pingpong[i].texture.view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(
                                            wgpu::Color {
                                                r: 0.0,
                                                g: 0.0,
                                                b: 0.0,
                                                a: 1.0,
                                            },
                                        ),
                                        store: true,
                                    },
                                },
                            )],
                            depth_stencil_attachment: None,
                        },
                    );

                    render_pass.set_pipeline(&self.gaussian_pipeline);
                    render_pass.set_bind_group(0, tex, &[]);
                    render_pass.set_bind_group(1, &self.lighting_params, &[]);
                    render_pass.set_bind_group(
                        2,
                        if horizontal {
                            &self.blur_direction_group_0
                        } else {
                            &self.blur_direction_group_1
                        },
                        &[],
                    );

                    render_pass.draw(0..3, 0..1);
                }

                horizontal = !horizontal;

                if first_iteration {
                    first_iteration = false;
                }
            }
        }

        let use_mipmaps = false;

        if use_mipmaps {
            self.mipmap_generator.generate_mipmaps(
                encoder,
                device,
                &self.threshold.render_texture.texture,
                BLOOM_MIP_LEVEL_COUNT,
            );

            for i in 0..BLOOM_MIP_LEVEL_COUNT {
                let mip_view =
                    self.threshold.render_texture.texture.create_view(
                        &wgpu::TextureViewDescriptor {
                            base_mip_level: i,
                            mip_level_count: Some(1),
                            ..Default::default()
                        },
                    );

                let mip_bind_group =
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(&format!("Bloom Blur Bind Group {}", i)),
                        layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(
                                    &mip_view,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(
                                    &self.threshold.render_texture.sampler,
                                ),
                            },
                        ],
                    });

                // TODO: get rid of tweaks later
                let blend_variant = tweak!(1);
                let constant_blend = tweak!(0.5);

                let blend = if blend_variant < 3 {
                    let settings = match blend_variant {
                        0 => BloomSettings::NATURAL,
                        1 => BloomSettings::SCREEN_BLUR,
                        2 => BloomSettings::OLD_SCHOOL,
                        _ => unreachable!(),
                    };

                    compute_blend_factor(
                        &settings,
                        i as f32,
                        BLOOM_MIP_LEVEL_COUNT as f32,
                    ) as f64
                } else {
                    constant_blend
                };

                draw_post_processing_output(
                    &format!("Bloom Blur {}", i),
                    encoder,
                    &self.mip_blur_pipeline,
                    &mip_bind_group,
                    &self.lighting_params,
                    &self.blur_texture.view,
                    i == 0,
                    Some(blend),
                );
            }
        }
    }

    pub fn blit_final(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &wgpu::TextureView,
        params: &GlobalLightingParams,
    ) {
        draw_post_processing_output(
            "Bloom Merge",
            encoder,
            &self.merge_pipeline,
            if GlobalParams::get_int("bloom_alg") == 0 {
                &self.blur_bind_group
            } else {
                &self.pingpong[0].bind_group
            },
            &self.lighting_params,
            output_view,
            false,
            Some(params.bloom_lerp as f64),
        );
    }
}


// {,
//     let mut encoder = self.device.simple_encoder("Bloom MipMaps");

// encoder.copy_texture_to_texture(
//     wgpu::ImageCopyTexture {
//         aspect: wgpu::TextureAspect::All,
//         texture: &self.bloom.threshold.render_texture.texture,
//         mip_level: 0,
//         origin: wgpu::Origin3d::ZERO,
//     },
//     wgpu::ImageCopyTexture {
//         aspect: wgpu::TextureAspect::All,
//         texture: &self.bloom.mipmaps.texture,
//         mip_level: 0,
//         origin: wgpu::Origin3d::ZERO,
//     },
//     wgpu::Extent3d {
//         width: self.config.width,
//         height: self.config.height,
//         depth_or_array_layers: 1,
//     },
// );
//
// let blur_texture = device.create_texture(&wgpu::TextureDescriptor {
//     label: Some("Bloom Blur Texture"),
//     size: wgpu::Extent3d {
//         width: config.width,
//         height: config.height,
//         depth_or_array_layers: 1,
//     },
//     mip_level_count: 1,
//     sample_count: 1,
//     dimension: wgpu::TextureDimension::D2,
//     format: wgpu::TextureFormat::Rgba32Float,
//     usage: wgpu::TextureUsages::TEXTURE_BINDING |
//         wgpu::TextureUsages::COPY_DST |
//         wgpu::TextureUsages::RENDER_ATTACHMENT,
//     view_formats: &[],
// });


/// Calculates blend intensities of blur pyramid levels
/// during the upsampling + compositing stage.
///
/// The function assumes all pyramid levels are upsampled and
/// blended into higher frequency ones using this function to
/// calculate blend levels every time. The final (highest frequency)
/// pyramid level in not blended into anything therefore this function
/// is not applied to it. As a result, the *mip* parameter of 0 indicates
/// the second-highest frequency pyramid level (in our case that is the
/// 0th mip of the bloom texture with the original image being the
/// actual highest frequency level).
///
/// Parameters:
/// * *mip* - the index of the lower frequency pyramid level (0 - max_mip, where 0 indicates highest frequency mip but not the highest frequency image).
/// * *max_mip* - the index of the lowest frequency pyramid level.
///
/// This function can be visually previewed for all values of *mip* (normalized) with tweakable
/// [`BloomSettings`] parameters on [Desmos graphing calculator](https://www.desmos.com/calculator/ncc8xbhzzl).
#[allow(clippy::doc_markdown)]
fn compute_blend_factor(
    bloom_settings: &BloomSettings,
    mip: f32,
    max_mip: f32,
) -> f32 {
    let mut lf_boost =
        (1.0 - (1.0 - (mip / max_mip))
            .powf(1.0 / (1.0 - bloom_settings.low_frequency_boost_curvature))) *
            bloom_settings.low_frequency_boost;
    let high_pass_lq = 1.0 -
        (((mip / max_mip) - bloom_settings.high_pass_frequency) /
            bloom_settings.high_pass_frequency)
            .clamp(0.0, 1.0);
    lf_boost *= match bloom_settings.composite_mode {
        BloomCompositeMode::EnergyConserving => 1.0 - bloom_settings.intensity,
        BloomCompositeMode::Additive => 1.0,
    };

    (bloom_settings.intensity + lf_boost) * high_pass_lq
}


/// Applies a bloom effect to an HDR-enabled 2d or 3d camera.
///
/// Bloom emulates an effect found in real cameras and the human eye,
/// causing halos to appear around very bright parts of the scene.
///
/// See also <https://en.wikipedia.org/wiki/Bloom_(shader_effect)>.
///
/// # Usage Notes
///
/// **Bloom is currently not compatible with WebGL2.**
///
/// Often used in conjunction with `bevy_pbr::StandardMaterial::emissive` for 3d meshes.
///
/// Bloom is best used alongside a tonemapping function that desaturates bright colors,
/// such as [`crate::tonemapping::Tonemapping::TonyMcMapface`].
///
/// Bevy's implementation uses a parametric curve to blend between a set of
/// blurred (lower frequency) images generated from the camera's view.
/// See <https://starlederer.github.io/bloom/> for a visualization of the parametric curve
/// used in Bevy as well as a visualization of the curve's respective scattering profile.
#[allow(clippy::doc_markdown)]
#[derive(Clone)]
pub struct BloomSettings {
    /// Controls the baseline of how much the image is scattered (default: 0.15).
    ///
    /// This parameter should be used only to control the strength of the bloom
    /// for the scene as a whole. Increasing it too much will make the scene appear
    /// blurry and over-exposed.
    ///
    /// To make a mesh glow brighter, rather than increase the bloom intensity,
    /// you should increase the mesh's `emissive` value.
    ///
    /// # In energy-conserving mode
    /// The value represents how likely the light is to scatter.
    ///
    /// The value should be between 0.0 and 1.0 where:
    /// * 0.0 means no bloom
    /// * 1.0 means the light is scattered as much as possible
    ///
    /// # In additive mode
    /// The value represents how much scattered light is added to
    /// the image to create the glow effect.
    ///
    /// In this configuration:
    /// * 0.0 means no bloom
    /// * > 0.0 means a proportionate amount of scattered light is added
    pub intensity: f32,

    /// Low frequency contribution boost.
    /// Controls how much more likely the light
    /// is to scatter completely sideways (low frequency image).
    ///
    /// Comparable to a low shelf boost on an equalizer.
    ///
    /// # In energy-conserving mode
    /// The value should be between 0.0 and 1.0 where:
    /// * 0.0 means low frequency light uses base intensity for blend factor calculation
    /// * 1.0 means low frequency light contributes at full power
    ///
    /// # In additive mode
    /// The value represents how much scattered light is added to
    /// the image to create the glow effect.
    ///
    /// In this configuration:
    /// * 0.0 means no bloom
    /// * > 0.0 means a proportionate amount of scattered light is added
    pub low_frequency_boost: f32,

    /// Low frequency contribution boost curve.
    /// Controls the curvature of the blend factor function
    /// making frequencies next to the lowest ones contribute more.
    ///
    /// Somewhat comparable to the Q factor of an equalizer node.
    ///
    /// Valid range:
    /// * 0.0 - base base intensity and boosted intensity are linearly interpolated
    /// * 1.0 - all frequencies below maximum are at boosted intensity level
    pub low_frequency_boost_curvature: f32,

    /// Tightens how much the light scatters (default: 1.0).
    ///
    /// Valid range:
    /// * 0.0 - maximum scattering angle is 0 degrees (no scattering)
    /// * 1.0 - maximum scattering angle is 90 degrees
    pub high_pass_frequency: f32,

    pub prefilter_settings: BloomPrefilterSettings,

    /// Controls whether bloom textures
    /// are blended between or added to each other. Useful
    /// if image brightening is desired and a must-change
    /// if `prefilter_settings` are used.
    ///
    /// # Recommendation
    /// Set to [`BloomCompositeMode::Additive`] if `prefilter_settings` are
    /// configured in a non-energy-conserving way,
    /// otherwise set to [`BloomCompositeMode::EnergyConserving`].
    pub composite_mode: BloomCompositeMode,
}

impl BloomSettings {
    /// The default bloom preset.
    pub const NATURAL: Self = Self {
        intensity: 0.15,
        low_frequency_boost: 0.7,
        low_frequency_boost_curvature: 0.95,
        high_pass_frequency: 1.0,
        prefilter_settings: BloomPrefilterSettings {
            threshold: 0.0,
            threshold_softness: 0.0,
        },
        composite_mode: BloomCompositeMode::EnergyConserving,
    };

    /// A preset that's similiar to how older games did bloom.
    pub const OLD_SCHOOL: Self = Self {
        intensity: 0.05,
        low_frequency_boost: 0.7,
        low_frequency_boost_curvature: 0.95,
        high_pass_frequency: 1.0,
        prefilter_settings: BloomPrefilterSettings {
            threshold: 0.6,
            threshold_softness: 0.2,
        },
        composite_mode: BloomCompositeMode::Additive,
    };

    /// A preset that applies a very strong bloom, and blurs the whole screen.
    pub const SCREEN_BLUR: Self = Self {
        intensity: 1.0,
        low_frequency_boost: 0.0,
        low_frequency_boost_curvature: 0.0,
        high_pass_frequency: 1.0 / 3.0,
        prefilter_settings: BloomPrefilterSettings {
            threshold: 0.0,
            threshold_softness: 0.0,
        },
        composite_mode: BloomCompositeMode::EnergyConserving,
    };
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self::NATURAL
    }
}

/// Applies a threshold filter to the input image to extract the brightest
/// regions before blurring them and compositing back onto the original image.
/// These settings are useful when emulating the 1990s-2000s game look.
///
/// # Considerations
/// * Changing these settings creates a physically inaccurate image
/// * Changing these settings makes it easy to make the final result look worse
/// * Non-default prefilter settings should be used in conjuction with [`BloomCompositeMode::Additive`]
#[derive(Default, Clone)]
pub struct BloomPrefilterSettings {
    /// Baseline of the quadratic threshold curve (default: 0.0).
    ///
    /// RGB values under the threshold curve will not contribute to the effect.
    pub threshold: f32,

    /// Controls how much to blend between the thresholded and non-thresholded colors (default: 0.0).
    ///
    /// 0.0 = Abrupt threshold, no blending
    /// 1.0 = Fully soft threshold
    ///
    /// Values outside of the range [0.0, 1.0] will be clamped.
    pub threshold_softness: f32,
}

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub enum BloomCompositeMode {
    EnergyConserving,
    Additive,
}
