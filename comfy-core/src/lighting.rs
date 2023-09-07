use crate::*;

static LIGHTS: Lazy<AtomicRefCell<LightingState>> =
    Lazy::new(|| AtomicRefCell::new(LightingState::default()));

#[derive(Default)]
pub struct LightingState {
    pub lights: Vec<Light>,
}

impl LightingState {
    pub fn begin_frame() {
        LIGHTS.borrow_mut().lights.clear();
    }

    pub fn take_lights() -> Vec<Light> {
        LIGHTS.borrow_mut().lights.clone()
    }
}

pub fn add_light(light: Light) {
    LIGHTS.borrow_mut().lights.push(light);
}

pub fn light_count() -> usize {
    LIGHTS.borrow().lights.len()
}

pub struct PointLight {
    pub radius: f32,
    pub radius_mod: f32,
    pub strength: f32,
    pub strength_mod: f32,
    pub color: Color,
}

#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Light {
    pub color: Color,
    pub world_position: [f32; 2],
    pub screen_position: [f32; 2],
    pub radius: f32,
    pub strength: f32,
    pub _padding: [f32; 2],
}

impl Light {
    pub fn simple(world_position: Vec2, radius: f32, strength: f32) -> Self {
        Self {
            color: WHITE,
            world_position: world_position.as_array(),
            screen_position: Vec2::ZERO.as_array(),
            radius,
            strength,
            _padding: [0.0; 2],
        }
    }
}

pub const MAX_LIGHTS: usize = 128;

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LightUniform {
    pub lights: [Light; 128],
    pub num_lights: i32,
    _padding: [f32; 3],
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            lights: [Light::default(); 128],
            num_lights: 0,
            _padding: [0.0; 3],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalLightingParams {
    pub ambient_light_color: [f32; 4],
    pub ambient_light_intensity: f32,
    pub quadratic_falloff: u32,
    pub lighting_enabled: u32,
    pub shadow_strength: f32,
    pub fog_color: [f32; 4],
    pub film_grain_strength: f32,
    pub noise_strength: f32,
    pub light_blending_mode: u32,
    pub global_light_intensity: f32,
    pub global_light_color: [f32; 4],
    pub vignette_color: [f32; 4],
    pub vignette_intensity: f32,
    pub vignette_radius: f32,
    pub shake_amount: f32,
    pub time: f32,
    pub color_balance: [f32; 4],
    pub global_brightness: f32,
    pub debug_visualization: u32,
    pub gamma_correction: f32,
    pub use_lut: u32,

    /// Exposure value (EV) offset, measured in stops.
    pub exposure: f32,

    /// Non-linear luminance adjustment applied before tonemapping. y = pow(x, gamma)
    pub gamma: f32,

    /// Saturation adjustment applied before tonemapping.
    /// Values below 1.0 desaturate, with a value of 0.0 resulting in a grayscale image
    /// with luminance defined by ITU-R BT.709.
    /// Values above 1.0 increase saturation.
    pub pre_saturation: f32,

    /// Saturation adjustment applied after tonemapping.
    /// Values below 1.0 desaturate, with a value of 0.0 resulting in a grayscale image
    /// with luminance defined by ITU-R BT.709
    /// Values above 1.0 increase saturation.
    pub post_saturation: f32,

    pub resolution: [f32; 2],
    pub chromatic_aberration: f32,
    pub bloom_threshold: f32,
    pub bloom_lerp: f32,
    pub bloom_gamma: f32,
    pub _padding: [f32; 2],
}

impl Default for GlobalLightingParams {
    fn default() -> Self {
        GlobalLightingParams {
            ambient_light_color: [1.0, 1.0, 1.0, 1.0],
            ambient_light_intensity: 1.0,
            quadratic_falloff: 0,
            lighting_enabled: 1,
            shadow_strength: 1.0,
            fog_color: [0.5, 0.5, 0.5, 1.0],
            film_grain_strength: 0.5,
            noise_strength: 0.2,
            light_blending_mode: 0,
            global_light_intensity: 1.0,
            global_light_color: [1.0, 1.0, 1.0, 1.0],
            vignette_color: [0.0, 0.0, 0.0, 1.0],
            vignette_intensity: 0.5,
            vignette_radius: 1.0,
            color_balance: [1.0, 1.0, 1.0, 1.0],
            global_brightness: 1.0,
            debug_visualization: 0,
            gamma_correction: 1.0,
            use_lut: 0,
            shake_amount: 0.0,
            time: 0.0,

            exposure: 0.0,
            gamma: 1.0,
            pre_saturation: 1.0,
            post_saturation: 1.0,

            resolution: [1920.0, 1080.0],
            chromatic_aberration: 0.0,
            bloom_threshold: 0.8,
            bloom_lerp: 0.3,
            bloom_gamma: 1.0,
            _padding: [0.0; 2],
        }
    }
}

pub fn lighting_ui(
    params: &mut GlobalLightingParams,
    ui: &mut egui::Ui,
) -> bool {
    let mut changed = false;

    changed |= field_editor(
        ui,
        "Chromatic Aberration",
        &mut params.chromatic_aberration,
    );

    changed |= field_editor(ui, "Bloom Threshold", &mut params.bloom_threshold);
    changed |= field_editor(ui, "Bloom Lerp", &mut params.bloom_lerp);
    changed |= field_editor(ui, "Bloom Gamma", &mut params.bloom_gamma);

    changed |= field_editor(ui, "Exposure", &mut params.exposure);
    changed |= field_editor(ui, "Gamma", &mut params.gamma);
    changed |= field_editor(ui, "Pre-Saturation", &mut params.pre_saturation);
    changed |= field_editor(ui, "Post-Saturation", &mut params.post_saturation);

    changed |= field_editor(ui, "Shake Amount", &mut params.shake_amount);

    changed |= field_editor_vec4(
        ui,
        "Ambient Light Color",
        &mut params.ambient_light_color,
    );
    changed |= field_editor(
        ui,
        "Ambient Light Intensity",
        &mut params.ambient_light_intensity,
    );
    changed |= field_editor(
        ui,
        "Quadratic Falloff",
        &mut params.quadratic_falloff,
    );

    changed |=
        field_editor(ui, "Light Blend Mode", &mut params.light_blending_mode);
    changed |= field_editor(
        ui,
        "Global Light Intensity",
        &mut params.global_light_intensity,
    );
    changed |= field_editor_vec4(
        ui,
        "Global Light Color",
        &mut params.global_light_color,
    );

    changed |=
        field_editor(ui, "Lighting Enabled", &mut params.lighting_enabled);
    changed |= field_editor(ui, "Shadow Strength", &mut params.shadow_strength);
    changed |= field_editor_vec4(ui, "Fog Color", &mut params.fog_color);
    changed |= field_editor(
        ui,
        "Film Grain Strength",
        &mut params.film_grain_strength,
    );
    changed |= field_editor(ui, "Noise Strength", &mut params.noise_strength);
    changed |= field_editor(
        ui,
        "Debug Visualization",
        &mut params.debug_visualization,
    );

    changed |=
        field_editor_vec4(ui, "Vignette Color", &mut params.vignette_color);
    changed |=
        field_editor(ui, "Vignette Intensity", &mut params.vignette_intensity);
    changed |= field_editor(ui, "Vignette Radius", &mut params.vignette_radius);
    changed |=
        field_editor_vec4(ui, "Color Balance", &mut params.color_balance);
    changed |=
        field_editor(ui, "Global Brightness", &mut params.global_brightness);
    changed |=
        field_editor(ui, "Gamma Correction", &mut params.gamma_correction);
    changed |= field_editor(ui, "Use LUT", &mut params.use_lut);

    changed
}

fn field_editor_vec4(
    ui: &mut egui::Ui,
    label: &str,
    field: &mut [f32; 4],
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.vertical(|ui| {
            let labels = ["X", "Y", "Z", "W"];
            ui.horizontal(|ui| {
                for (i, value) in field.iter_mut().enumerate() {
                    let value_changed = ui
                        .add(
                            egui::DragValue::new(value)
                                .speed(0.01)
                                .clamp_range(0.0..=1.0)
                                .prefix(labels[i]),
                        )
                        .changed();
                    changed |= value_changed;
                }
            });
        });
    });
    changed
}

fn field_editor<T>(ui: &mut egui::Ui, label: &str, field: &mut T) -> bool
where T: Clone + PartialEq + 'static + egui::emath::Numeric {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(label);
        changed = ui.add(egui::DragValue::new(field).speed(0.01)).changed();
    });

    changed
}
