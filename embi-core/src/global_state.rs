use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::*;

static FRAME_TIME: AtomicU32 =
    AtomicU32::new(unsafe { std::mem::transmute(1.0f32) });

static DELTA: AtomicU32 =
    AtomicU32::new(unsafe { std::mem::transmute(1f32 / 60f32) });

static TIME_SCALE: AtomicU32 =
    AtomicU32::new(unsafe { std::mem::transmute(1.0f32) });

static TIME: AtomicU64 = AtomicU64::new(unsafe { std::mem::transmute(0.0f64) });

static UNPAUSED_TIME: AtomicU64 =
    AtomicU64::new(unsafe { std::mem::transmute(0.0f64) });

pub fn frame_time() -> f32 {
    f32::from_bits(FRAME_TIME.load(Ordering::SeqCst))
}

pub fn set_frame_time(value: f32) {
    FRAME_TIME.store(value.to_bits(), Ordering::SeqCst);
}

pub fn time_scale() -> f32 {
    f32::from_bits(TIME_SCALE.load(Ordering::SeqCst))
}

pub fn set_time_scale(value: f32) {
    TIME_SCALE.store(value.to_bits(), Ordering::SeqCst);
}

pub fn delta() -> f32 {
    f32::from_bits(DELTA.load(Ordering::SeqCst)) * time_scale()
}

pub fn set_delta(value: f32) {
    DELTA.store(value.to_bits(), Ordering::SeqCst);
}

pub fn get_time() -> f64 {
    f64::from_bits(TIME.load(Ordering::SeqCst))
}

pub fn set_time(value: f64) {
    TIME.store(value.to_bits(), Ordering::SeqCst);
}

pub fn get_unpaused_time() -> f64 {
    f64::from_bits(UNPAUSED_TIME.load(Ordering::SeqCst))
}

pub fn set_unpaused_time(value: f64) {
    UNPAUSED_TIME.store(value.to_bits(), Ordering::SeqCst);
}

pub static GLOBAL_STATE: Lazy<AtomicRefCell<GlobalState>> =
    Lazy::new(|| AtomicRefCell::new(GlobalState::default()));

#[derive(Default)]
pub struct GlobalState {
    pub mouse_wheel: (f32, f32),
    pub mouse_position: Vec2,
    pub mouse_rel: IVec2,
    pub mouse_world: Vec2,

    pub mouse_locked: bool,

    pub egui_scale_factor: f32,

    pub frame: u32,
    pub fps: i32,

    pub mesh_queue: Vec<MeshDraw>,
    pub text_queue: Vec<DrawText>,

    pub clear_color: Color,

    pub screen_size: Vec2,

    pub pressed: HashSet<KeyCode>,
    pub just_pressed: HashSet<KeyCode>,
    pub just_released: HashSet<KeyCode>,

    pub mouse_pressed: HashSet<MouseButton>,
    pub mouse_just_pressed: HashSet<MouseButton>,
    pub mouse_just_released: HashSet<MouseButton>,

    pub play_sound_queue: Vec<Sound>,
    pub stop_sound_queue: Vec<Sound>,
}
