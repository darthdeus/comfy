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
    let mut camera = main_camera_mut();

    camera.shake_timer = timer;
    camera.shake_amount = amount;
}

pub fn main_camera() -> AtomicRef<'static, MainCamera> {
    MAIN_CAMERA.borrow()
}

pub fn main_camera_mut() -> AtomicRefMut<'static, MainCamera> {
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

#[derive(Copy, Clone)]
struct HistoryStackValue {
    pub center: Vec2,
    pub desired_zoom: f32,
    pub zoom: f32,
}

pub type CameraMatrixFn =
    Box<dyn (Fn(&MainCamera, Vec2) -> Mat4) + Send + Sync + 'static>;

pub struct MainCamera {
    /// Screenshake time remaining.
    pub shake_timer: f32,
    /// Amount of screenshake to apply.
    pub shake_amount: f32,

    pub recoil: f32,

    /// Center of the camera. This updates the camera immediately for the current frame without any
    /// smoothing. If you need something set `target` instead.
    pub center: Vec2,
    /// Smoothing target for the camera. By default this also uses a deadzone as defined by
    /// `deadzone_width` and `deadzone_height`. If you don't want a deadzone, set those to zero.
    pub target: Option<Vec2>,

    /// Smoothing speed when `target` is set.
    pub smoothing_speed: f32,
    /// Width of the camera deadzone in world space.
    pub deadzone_width: f32,
    /// Height of the camera deadzone in world space.
    pub deadzone_height: f32,

    pub aspect_ratio: f32,

    pub zoom: f32,
    pub desired_zoom: f32,

    /// Optional camera matrix function that allows the user to create their own projection matrix.
    ///
    /// See the implementation of `build_view_projection_matrix` for what is the default with
    /// `Mat4::orthographic_rh`. Note that this doesn't have to return an orthographic perspective
    /// matrix, it can be anything (perspective projection, etc.).
    pub matrix_fn: Option<CameraMatrixFn>,
    /// Override config allowing to disable matrix_fn even when one is provided.
    /// Useful for debugging.
    pub use_matrix_fn: bool,

    history_stack: Vec<HistoryStackValue>,
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
            target: None,

            deadzone_width: 3.0,
            deadzone_height: 2.0,

            smoothing_speed: 4.0,

            aspect_ratio: 1.0,

            zoom,
            desired_zoom: zoom,

            history_stack: Vec::new(),

            matrix_fn: None,
            use_matrix_fn: true,
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.shake_timer -= delta;
        self.shake_timer = self.shake_timer.max(0.0);
        self.recoil = (self.recoil - delta).max(0.0);

        set_px(self.zoom / screen_width());

        if let Some(player_position) = self.target {
            let deadzone_hw = self.deadzone_width / 2.0;
            let deadzone_hh = self.deadzone_height / 2.0;

            let ox = player_position.x - self.center.x;
            let oy = player_position.y - self.center.y;

            let dx = ox.abs() - deadzone_hw;
            let dy = oy.abs() - deadzone_hh;

            let mut offset = Vec2::ZERO;

            if dx > 0.0 {
                offset.x = dx * ox.signum();
            }

            if dy > 0.0 {
                offset.y = dy * oy.signum();
            }

            self.center += offset * delta * self.smoothing_speed;
            self.center = player_position;
        }
    }

    pub fn push_center(&mut self, new_center: Vec2, new_zoom: f32) {
        self.history_stack.push(HistoryStackValue {
            center: self.center,
            desired_zoom: self.desired_zoom,
            zoom: self.zoom,
        });

        self.center = new_center;

        self.desired_zoom = new_zoom;
        self.zoom = new_zoom;
    }

    pub fn pop_center(&mut self) {
        if let Some(item) = self.history_stack.pop() {
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

    pub fn shake(&mut self, amount: f32, time: f32) {
        self.shake_amount = amount;
        self.shake_timer = time;
    }

    pub fn current_shake(&self) -> f32 {
        self.shake_amount * self.shake_timer.clamp(0.0, 1.0)
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let hx = self.zoom / 2.0;
        let hy = self.zoom / 2.0 / self.aspect_ratio;

        let range = 1000.0;

        const SHAKE: f32 = 0.2;

        let (sx, sy) = if self.shake_timer > 0.0 {
            let off = random_around(Vec2::ZERO, 0.0, SHAKE * self.shake_amount);
            (off.x, off.y)
        } else {
            (0.0, 0.0)
        };

        let center = self.center + vec2(sx, sy);

        let ortho_camera = Mat4::orthographic_rh(
            center.x - hx,
            center.x + hx,
            center.y - hy,
            center.y + hy,
            -range,
            range,
        );

        if let Some(matrix_fn) = self.matrix_fn.as_ref() {
            if self.use_matrix_fn {
                matrix_fn(self, center)
            } else {
                ortho_camera
            }
        } else {
            ortho_camera
        }
    }

    pub fn screen_to_world(&self, position: Vec2) -> Vec2 {
        let viewport = self.world_viewport();
        let camera_center = self.center;

        let state = GLOBAL_STATE.borrow();

        let normalized = position / state.screen_size;
        let normalized = vec2(normalized.x, 1.0 - normalized.y);

        let world_unoffset = (normalized - 0.5) * viewport;

        world_unoffset + camera_center
    }

    pub fn world_to_screen(&self, position: Vec2) -> Vec2 {
        let viewport = self.world_viewport();

        let position = position - self.center;

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

    pub fn world_to_render_px(
        &self,
        position: Vec2,
        render_scale: f32,
    ) -> IVec2 {
        let px = self.world_to_screen(position).as_ivec2();
        (px.as_vec2() * render_scale).as_ivec2()
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

// TODO: get rid of this
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
