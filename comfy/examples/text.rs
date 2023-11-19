use comfy::*;

simple_game!("Text Example", GameState, setup, update);

pub struct GameState {
    pub fonts: Vec<FontHandle>,
    pub font_size: f32,
}

impl GameState {
    pub fn new(_c: &mut EngineState) -> Self {
        // let font_data =
        //     include_bytes!("../assets/ArianaVioleta.ttf") as &[u8];
        // let font_data =
        //     include_bytes!("../../assets/ThaleahFat_TTF.ttf") as &[u8];

        Self {
            fonts: vec![
                load_font_from_bytes(include_bytes!(
                    "../../assets/fonts/Orbitron-Black.ttf"
                )),
                load_font_from_bytes(include_bytes!(
                    "../../assets/fonts/Orbitron-Regular.ttf"
                )),
            ],
            font_size: 32.0,
        }
    }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {
    game_config_mut().bloom_enabled = true;
}

fn update(state: &mut GameState, _c: &mut EngineContext) {
    clear_background(DARKBLUE);

    egui::Window::new("Font Controls")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -160.0))
        .show(egui(), |ui| {
            ui.add(egui::Slider::new(&mut state.font_size, 12.0..=80.0));
        });

    draw_text_pro_experimental(
        simple_styled_text("comfy has *c*o*m*f*y *t*e*x*t rendering"),
        // mouse_world(),
        vec2(0.0, 1.0),
        WHITE,
        TextAlign::Center,
        state.font_size,
        if get_time() as i32 % 2 == 1 {
            state.fonts[1]
        } else {
            state.fonts[0]
        }, // *state.fonts.choose().unwrap(),
    );

    draw_text_ex(
        "with both builtin TTF rasterizer and with egui",
        vec2(0.0, -1.0),
        TextAlign::Center,
        TextParams {
            color: YELLOW,
            // Use egui fonts
            font: egui::FontId::new(
                state.font_size,
                egui::FontFamily::Proportional,
            ),
            ..Default::default()
        },
    );
}
