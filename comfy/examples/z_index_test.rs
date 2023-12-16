use comfy::*;

simple_game!("Z-index test", GameState, setup, update);

pub struct GameState {
    pub font: FontHandle,
    pub font_size: f32,
}

impl GameState {
    pub fn new(_c: &mut EngineState) -> Self {
        Self {
            font: load_font_from_bytes(include_bytes!(
                "../../assets/fonts/Orbitron-Black.ttf"
            )),
            font_size: 32.0,
        }
    }
}


const Z_BG: i32 = 0;
const Z_1: i32 = 1;
const Z_2: i32 = 2;

fn setup(_state: &mut GameState, _c: &mut EngineContext) {
    set_y_sort(0, true);
}

fn update(state: &mut GameState, _c: &mut EngineContext) {
    draw_rect(Vec2::ZERO, splat(40.0), GREEN.darken(0.99), Z_BG);

    {
        let left = vec2(-8.0, 0.0);

        draw_comfy(left, WHITE, Z_1, splat(2.0));
        draw_comfy(
            left + vec2(1.0, (4.0 * get_time()).sin() as f32),
            RED,
            Z_1,
            splat(3.0),
        );
    }

    {
        for (n, i) in (-5i32..5).enumerate() {
            let colors = [RED, BLUE];

            let index = (i.rem_euclid(colors.len() as i32)) as usize;

            let pos = vec2(-1.0 + i as f32 * 0.1, i as f32 * 0.8);

            draw_circle(pos, 0.5, colors[index], Z_2);

            draw_text_pro_experimental(
                simple_styled_text(&format!("{}", n)),
                pos,
                WHITE,
                TextAlign::Center,
                32.0,
                state.font,
                100,
            );

            draw_circle(
                vec2(2.0 - i as f32 * 0.1, -i as f32 * 0.8),
                0.5,
                colors[index],
                Z_2,
            );
        }
    }
}
