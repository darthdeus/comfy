use comfy::*;

simple_game!("Text Example", GameState, setup, update);

pub struct GameState {
    pub fonts: Vec<FontHandle>,
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
        }
    }
}

fn setup(_state: &mut GameState, _c: &mut EngineContext) {
    game_config_mut().bloom_enabled = true;
}

fn update(state: &mut GameState, _c: &mut EngineContext) {
    clear_background(DARKBLUE);

    draw_text_pro_experimental(
        simple_styled_text("comfy has *c*o*m*f*y *t*e*x*t rendering"),
        vec2(-5.0, 1.0),
        WHITE,
        TextAlign::Center,
        16.0,
        *state.fonts.choose().unwrap(),
    );

    draw_text_ex(
        "with both builtin TTF rasterizer and with egui",
        vec2(0.0, -1.0),
        TextAlign::Center,
        TextParams {
            color: YELLOW,
            // Use egui fonts
            font: egui::FontId::new(32.0, egui::FontFamily::Proportional),
            ..Default::default()
        },
    );
}
