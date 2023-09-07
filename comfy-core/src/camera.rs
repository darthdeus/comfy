use std::sync::atomic::{AtomicU32, Ordering};

use crate::*;

pub static MAIN_CAMERA: Lazy<AtomicRefCell<MainCamera>> =
    Lazy::new(|| AtomicRefCell::new(MainCamera::default()));

pub const LINE_W: f32 = 2.0;

static PX: AtomicU32 =
    AtomicU32::new(unsafe { std::mem::transmute(0.0347f32) });

pub fn set_px(value: f32) {
    PX.store(value.to_bits(), Ordering::SeqCst);
}

pub fn px() -> f32 {
    f32::from_bits(PX.load(Ordering::SeqCst))
}

pub fn mouse_screen() -> Vec2 {
    let pos = GLOBAL_STATE.borrow().mouse_position;
    Vec2::new(pos.x, pos.y)
}

pub fn mouse_world() -> Vec2 {
    GLOBAL_STATE.borrow().mouse_world
}

pub fn screen_width() -> f32 {
    GLOBAL_STATE.borrow().screen_size.x
}

pub fn aspect_ratio() -> f32 {
    MAIN_CAMERA.borrow().aspect_ratio
}

pub fn screen_height() -> f32 {
    GLOBAL_STATE.borrow().screen_size.y
}

pub fn screenshake(timer: f32, amount: f32) {
    let mut camera = main_camera();

    camera.shake_timer = timer;
    camera.shake_amount = amount;
}

pub fn main_camera() -> AtomicRefMut<'static, MainCamera> {
    MAIN_CAMERA.borrow_mut()
}

pub fn set_main_camera_zoom(zoom: f32) {
    MAIN_CAMERA.borrow_mut().zoom = zoom;
}


pub fn egui_scale_factor() -> f32 {
    GLOBAL_STATE.borrow().egui_scale_factor
}

pub fn world_to_gl_screen(position: Vec2) -> Vec2 {
    let mut screen = world_to_screen(position);
    screen.y = screen_height() - screen.y;
    screen
}

pub fn world_to_screen(position: Vec2) -> Vec2 {
    main_camera().world_to_screen(position)
}

pub fn screen_to_world(position: Vec2) -> Vec2 {
    main_camera().screen_to_world(position)
}

// TODO: use this for zoom
pub struct DampedSpring {
    pub value: f32,
    pub target: f32,
    pub velocity: f32,
    pub damping: f32,
}

impl DampedSpring {
    pub fn new(target: f32, damping: f32) -> DampedSpring {
        DampedSpring { value: target, target, velocity: 0.0, damping }
    }

    pub fn update(&mut self) {
        let diff = self.target - self.value;
        self.velocity += diff * self.damping;
        self.value += self.velocity;
        self.velocity *= 0.9; // This is to ensure the value will eventually settle
    }
}

#[derive(Clone)]
pub struct MainCamera {
    pub shake_timer: f32,
    pub shake_amount: f32,

    pub recoil: f32,

    pub center: Vec2,
    pub desired_center: Vec2,
    pub target: Option<Vec2>,

    pub smoothing_speed: f32,

    pub aspect_ratio: f32,

    pub zoom: f32,
    pub desired_zoom: f32,

    history_stack: Vec<HistoryStackValue>,
}

#[derive(Copy, Clone)]
struct HistoryStackValue {
    pub center: Vec2,
    pub desired_center: Vec2,
    pub desired_zoom: f32,
    pub zoom: f32,
}

impl Default for MainCamera {
    fn default() -> Self {
        Self::new(Vec2::new(0.0, 0.0), 30.0)
    }
}

impl MainCamera {
    pub fn new(center: Vec2, zoom: f32) -> Self {
        Self {
            shake_timer: 0.0,
            shake_amount: 0.0,

            recoil: 0.0,

            center,
            desired_center: center,
            target: None,

            smoothing_speed: 2.0,

            aspect_ratio: 1.0,

            zoom,
            desired_zoom: zoom,

            history_stack: Vec::new(),
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.shake_timer -= delta;
        self.shake_timer = self.shake_timer.max(0.0);
        self.recoil = (self.recoil - delta).max(0.0);

        set_px(self.zoom / screen_width());

        if let Some(player_position) = self.target {
            const DEADZONE_WIDTH: f32 = 3.0;
            const DEADZONE_HEIGHT: f32 = 2.0;

            let dx = (player_position.x - self.center.x).abs();
            let dy = (player_position.y - self.center.y).abs();

            if dx > DEADZONE_WIDTH / 2.0 || dy > DEADZONE_HEIGHT / 2.0 {
                let mut new_center = self.center;

                if dx > DEADZONE_WIDTH / 2.0 {
                    new_center.x = player_position.x -
                        DEADZONE_WIDTH / 2.0 * player_position.x.signum();
                }

                if dy > DEADZONE_HEIGHT / 2.0 {
                    new_center.y = player_position.y -
                        DEADZONE_HEIGHT / 2.0 * player_position.y.signum();
                }

                self.center = self.center +
                    (new_center - self.center) * self.smoothing_speed * delta;
            }
        }

        // if let Some(player_position) = self.target {
        //     const DEADZONE_WIDTH: f32 = 3.0;
        //     const DEADZONE_HEIGHT: f32 = 2.0;
        //
        //     let dx = (player_position.x - self.center.x).abs();
        //     let dy = (player_position.y - self.center.y).abs();
        //
        //     let mut new_center = self.center;
        //
        //     if dx > DEADZONE_WIDTH / 2.0 {
        //         new_center.x = player_position.x -
        //             DEADZONE_WIDTH / 2.0 *
        //                 (player_position.x - self.center.x).signum();
        //     }
        //
        //     if dy > DEADZONE_HEIGHT / 2.0 {
        //         new_center.y = player_position.y -
        //             DEADZONE_HEIGHT / 2.0 *
        //                 (player_position.y - self.center.y).signum();
        //     }
        //
        //     self.center = new_center;
        // }
    }

    pub fn push_center(&mut self, new_center: Vec2, new_zoom: f32) {
        self.history_stack.push(HistoryStackValue {
            desired_center: self.desired_center,
            center: self.center,
            desired_zoom: self.desired_zoom,
            zoom: self.zoom,
        });

        self.desired_center = new_center;
        self.center = new_center;

        self.desired_zoom = new_zoom;
        self.zoom = new_zoom;
    }

    pub fn pop_center(&mut self) {
        if let Some(item) = self.history_stack.pop() {
            self.desired_center = item.desired_center;
            self.center = item.center;
            self.zoom = item.zoom;
            self.desired_zoom = item.desired_zoom;
        }
    }

    pub fn bump_recoil(&mut self, amount: f32) {
        self.recoil = (self.recoil + amount).clamp(0.0, 1.0);
    }

    pub fn world_viewport(&self) -> Vec2 {
        vec2(self.zoom, self.zoom / self.aspect_ratio)
    }

    pub fn screen_top_left(&self) -> Vec2 {
        let world_viewport = self.world_viewport();
        self.center + vec2(-world_viewport.x, world_viewport.y) / 2.0
    }

    pub fn screen_top_right(&self) -> Vec2 {
        let world_viewport = self.world_viewport();
        self.center + vec2(world_viewport.x, world_viewport.y) / 2.0
    }

    pub fn current_shake(&self) -> f32 {
        self.shake_amount * self.shake_timer.clamp(0.0, 1.0)
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let hx = self.zoom / 2.0;
        let hy = self.zoom / 2.0 / self.aspect_ratio;

        let range = 500.0;

        const SHAKE: f32 = 0.2;

        let (sx, sy) = if self.shake_timer > 0.0 {
            let off = random_around(Vec2::ZERO, 0.0, SHAKE * self.shake_amount);
            (off.x, off.y)
        } else {
            (0.0, 0.0)
        };

        let center = self.center + vec2(sx, sy);

        Mat4::orthographic_rh(
            // let proj = Mat4::orthographic_lh(
            center.x - hx,
            center.x + hx,
            center.y - hy,
            center.y + hy,
            -range,
            range,
        )
    }

    pub fn screen_to_world(&self, position: Vec2) -> Vec2 {
        let camera = main_camera();
        let viewport = camera.world_viewport();
        let camera_center = camera.center;

        let state = GLOBAL_STATE.borrow();

        let normalized = position / state.screen_size;
        let normalized = vec2(normalized.x, 1.0 - normalized.y);

        let world_unoffset = (normalized - 0.5) * viewport;

        world_unoffset + camera_center
    }

    pub fn world_to_screen(&self, position: Vec2) -> Vec2 {
        let camera = main_camera();
        let viewport = camera.world_viewport();

        let position = position - camera.center;

        let state = GLOBAL_STATE.borrow();

        let moved = position + viewport / 2.0;
        let normalized = moved / viewport;
        let normalized = vec2(normalized.x, 1.0 - normalized.y);

        normalized * state.screen_size
        // vec2(
        //     normalized.x * state.screen_size.x / state.egui_scale_factor,
        //     normalized.y * state.screen_size.y / state.egui_scale_factor,
        // )
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Position {
    World { x: f32, y: f32 },
    Screen { x: ScreenVal, y: ScreenVal },
}

impl Position {
    pub fn world(x: f32, y: f32) -> Self {
        Self::World { x, y }
    }

    pub fn screen(x: ScreenVal, y: ScreenVal) -> Self {
        Self::Screen { x, y }
    }

    pub fn screen_px(x: f32, y: f32) -> Self {
        Self::Screen { x: ScreenVal::Px(x), y: ScreenVal::Px(y) }
    }

    // TODO: rename to screen & percent
    pub fn screen_percent(x: f32, y: f32) -> Self {
        Self::Screen { x: ScreenVal::Percent(x), y: ScreenVal::Percent(y) }
    }

    pub fn vec2(self) -> Vec2 {
        match self {
            Position::World { x, y } => vec2(x, y),
            Position::Screen { x, y } => vec2(x.to_f32(), y.to_f32()),
        }
    }

    pub fn to_world(self) -> Vec2 {
        match self {
            Position::World { x, y } => vec2(x, y),
            Position::Screen { x, y } => {
                screen_to_world(vec2(
                    x.to_px(screen_width()),
                    y.to_px(screen_height()),
                ))
            }
        }
    }

    pub fn to_screen(self) -> Vec2 {
        match self {
            Position::World { x, y } => world_to_screen(vec2(x, y)),
            Position::Screen { x, y } => {
                vec2(x.to_px(screen_width()), y.to_px(screen_height()))
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ScreenVal {
    Px(f32),
    Percent(f32),
}

impl ScreenVal {
    pub fn to_f32(self) -> f32 {
        match self {
            ScreenVal::Px(val) => val,
            ScreenVal::Percent(val) => val,
        }
    }

    pub fn to_px(self, screen_dim: f32) -> f32 {
        match self {
            ScreenVal::Px(val) => val,
            ScreenVal::Percent(val) => screen_dim * val,
        }
    }
}

// TODO: get rid fo this
#[derive(Copy, Clone, Debug)]
pub enum Value {
    World(f32),
    Px(f32),
    Percent(f32),
}

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    X,
    Y,
}

impl Value {
    pub fn to_world(self, axis: Axis) -> f32 {
        let viewport = main_camera().world_viewport();

        match self {
            Value::World(val) => val,
            Value::Px(val) => {
                match axis {
                    Axis::X => val / screen_width() * viewport.x,
                    Axis::Y => val / screen_height() * viewport.y,
                }
            }
            // TODO: simplify
            Value::Percent(percent) => {
                match axis {
                    Axis::X => percent * viewport.x,
                    Axis::Y => percent * viewport.y,
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub width: Value,
    pub height: Value,
}

impl Size {
    pub fn world(width: f32, height: f32) -> Self {
        Size { width: Value::World(width), height: Value::World(height) }
    }

    pub fn screen(width: f32, height: f32) -> Self {
        Size { width: Value::Px(width), height: Value::Px(height) }
    }

    pub fn percent(width: f32, height: f32) -> Self {
        Size { width: Value::Percent(width), height: Value::Percent(height) }
    }

    pub fn to_world(self) -> Vec2 {
        vec2(self.width.to_world(Axis::X), self.height.to_world(Axis::Y))
    }
}
