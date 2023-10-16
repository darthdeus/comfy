use comfy::*;

simple_game!("Colors", setup, update);

fn setup(c: &mut EngineContext) {
    c.load_texture_from_bytes(
        "gradients",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/gradients.png"
        )),
    );

    c.load_texture_from_bytes(
        "gray",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/gray-50percent.png"
        )),
    );
}

static COMFY_LOOK: AtomicRefCell<bool> = AtomicRefCell::new(false);

fn update(_c: &mut EngineContext) {
    clear_background(DARKGRAY);

    egui::Window::new("Color palette")
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 60.0))
        .show(egui(), |ui| {
            ui.checkbox(&mut COMFY_LOOK.borrow_mut(), "Comfy look");
        });

    pub const RGB_RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
    pub const RGB_GREEN: Color = Color::new(0.0, 1.0, 0.0, 1.0);
    pub const RGB_BLUE: Color = Color::new(0.0, 0.0, 1.0, 1.0);
    pub const RGB_YELLOW: Color = Color::new(1.0, 1.0, 0.0, 1.0);

    let colors = if *COMFY_LOOK.borrow() {
        [RED, GREEN, BLUE, YELLOW]
    } else {
        [RGB_RED, RGB_GREEN, RGB_BLUE, RGB_YELLOW]
    };

    let off = vec2(0.0, 1.0);

    draw_gradient(off + vec2(0.0, 0.0), BLACK, WHITE);
    draw_gradient(off + vec2(0.0, 1.0), BLACK, colors[0]);
    draw_gradient(off + vec2(0.0, 2.0), BLACK, colors[1]);
    draw_gradient(off + vec2(0.0, 3.0), BLACK, colors[2]);
    draw_gradient(off + vec2(0.0, 4.0), BLACK, colors[3]);

    draw_sprite(
        texture_id("gradients"),
        vec2(0.0, -3.0),
        WHITE,
        1,
        vec2(10.0, 5.0),
    );

    draw_rect(vec2(8.0, 0.5), splat(1.0), Color::rgb(0.5, 0.5, 0.5), 0);
    draw_text("Rectangle", vec2(10.0, 0.5), BLACK, TextAlign::Center);

    draw_sprite(texture_id("gray"), vec2(8.0, -0.5), WHITE, 0, splat(1.0));
    draw_text("Sprite", vec2(10.0, -0.5), BLACK, TextAlign::Center);
}

fn draw_gradient(pos: Vec2, left: Color, right: Color) {
    let hw = 5.0;
    let hh = 0.5;

    {
        let pos = pos.extend(0.0);

        draw_mesh(Mesh {
            vertices: vec![
                SpriteVertex::new(pos + vec3(-hw, hh, 0.0), Vec2::ZERO, left),
                SpriteVertex::new(pos + vec3(-hw, -hh, 0.0), Vec2::ZERO, left),
                SpriteVertex::new(pos + vec3(hw, hh, 0.0), Vec2::ZERO, right),
                SpriteVertex::new(pos + vec3(hw, -hh, 0.0), Vec2::ZERO, right),
            ]
            .into(),
            indices: vec![0, 2, 3, 0, 3, 1].into(),
            // Comfy doesn't do backface culling, this would also work.
            // indices: vec![3, 2, 0, 1, 3, 0].into(),
            texture: None,
            z_index: 0,
        });
    }

    let box_size = splat(2.0 * hh);
    let off = 0.2;

    draw_rect(pos - vec2(hw + hh + off, 0.0), box_size, left, 0);
    // draw_rect_outline(pos - vec2(hw + hh + off, 0.0), box_size, 0.1, BLACK, 1);

    draw_rect(pos + vec2(hw + hh + off, 0.0), box_size, right, 0);
    // draw_rect_outline(pos + vec2(hw + hh + off, 0.0), box_size, 0.1, LIGHTGRAY, 1);
}
