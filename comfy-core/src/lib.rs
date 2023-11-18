#![allow(clippy::uninlined_format_args)]
#![allow(clippy::new_without_default)]

mod asset_loader;
mod assets;
mod audio;
mod blood_canvas;
mod camera;
mod config;
#[cfg(not(target_arch = "wasm32"))]
mod desktop;
mod errors;
mod events;
mod fast_sprite;
mod global_state;
#[cfg(not(target_arch = "wasm32"))]
mod input;
mod lighting;
mod math;
mod perf_counters;
mod quad;
pub mod random;
mod shaders;
pub mod spatial_hash;
mod task_timer;
mod text;
mod timer;
mod tween;

pub use crate::asset_loader::*;
pub use crate::assets::*;
pub use crate::audio::*;
pub use crate::blood_canvas::*;
pub use crate::camera::*;
pub use crate::config::*;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::desktop::*;
pub use crate::errors::*;
pub use crate::events::*;
pub use crate::fast_sprite::*;
pub use crate::global_state::*;
pub use crate::input::*;
pub use crate::lighting::*;
pub use crate::math::*;
pub use crate::perf_counters::*;
pub use crate::quad::*;
pub use crate::random::*;
pub use crate::shaders::*;
pub use crate::task_timer::*;
pub use crate::text::*;
pub use crate::timer::*;
pub use crate::tween::*;

pub use std::any::Any;
pub use std::collections::VecDeque;
pub use std::hash::{Hash, Hasher};
pub use std::num::NonZeroU32;

pub const Z_BLOOD_CANVAS: i32 = 4;

pub use std::ops::DerefMut;

use std::ops::Add;
pub use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    f32::consts::PI,
    ops::{Mul, Range},
    rc::Rc,
    sync::Arc,
};

use num_traits::NumCast;
pub use rand::seq::SliceRandom;

pub use smallvec::{self, SmallVec};

pub use anyhow;
pub use anyhow::{bail, Result};

pub use bimap::BiHashMap;
pub use fxhash;
pub use num_traits;

#[cfg(target_arch = "wasm32")]
pub use instant::{Duration, Instant};
#[cfg(not(target_arch = "wasm32"))]
pub use notify;
#[cfg(not(target_arch = "wasm32"))]
pub use std::time::{Duration, Instant};

pub use inline_tweak;
pub use inline_tweak::tweak;

pub use std::future::Future;
pub use std::pin::Pin;
pub use std::task::Poll;

pub use hecs;
pub use hecs::{CommandBuffer, DynamicBundle, Entity, World};
pub use simple_easing::*;

pub use backtrace;
pub use backtrace::Backtrace;

pub use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
pub use bytemuck;
pub use cfg_if::cfg_if;
pub use egui;
pub use egui_plot;
pub use egui_winit;
pub use env_logger;
pub use epaint;
pub use glam::{
    ivec2, uvec2, vec2, vec3, vec4, Affine2, IVec2, Mat3, Mat4, UVec2, Vec2,
    Vec2Swizzles, Vec3, Vec4,
};
pub use image;
pub use image::DynamicImage;
pub use itertools::Itertools;
pub use log;
pub use log::{debug, error, info, trace, warn};
pub use once_cell::{
    self,
    sync::{Lazy, OnceCell},
};
pub use parking_lot::Mutex;
pub use rand::{distributions::uniform::SampleUniform, Rng, RngCore};

#[cfg(feature = "blobs")]
pub use blobs;

#[cfg(all(feature = "memory-stats", not(target_arch = "wasm32")))]
pub use memory_stats;

pub use num_complex::Complex;

#[cfg(feature = "tracy")]
pub use tracy_client;
pub use winit::{
    self,
    event::{
        ElementState, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode,
        WindowEvent,
    },
    window::Window,
};

pub use thunderdome::{Arena, Index};

pub use maplit::hashmap;

pub const FHD_RATIO: f32 = 1920.0 / 1080.0;

pub static GLOBAL_PARAMS: Lazy<AtomicRefCell<GlobalParams>> =
    Lazy::new(|| AtomicRefCell::new(GlobalParams::new()));

#[cfg(all(feature = "memprof", feature = "tracy"))]
#[global_allocator]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
    tracy_client::ProfiledAllocator::new(std::alloc::System, 100);

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
pub use jemalloc_ctl;

pub use ::rand::{
    distributions::Distribution, distributions::WeightedIndex,
    seq::IteratorRandom, thread_rng,
};

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_futures;
#[cfg(target_arch = "wasm32")]
pub use web_sys;

#[cfg(target_arch = "wasm32")]
pub use console_error_panic_hook;
#[cfg(target_arch = "wasm32")]
pub use console_log;

pub use anymap::AnyMap;
pub use bitflags::bitflags;
pub use crossbeam;
pub use lazy_static::lazy_static;
pub use ordered_float::OrderedFloat;

pub use comfy_color_backtrace as color_backtrace;
#[cfg(feature = "git-version")]
pub use comfy_git_version as git_version;
pub use comfy_include_dir as include_dir;

pub use kira;
pub use kira::manager::{AudioManager, AudioManagerSettings};
pub use kira::sound::static_sound::{
    StaticSoundData, StaticSoundHandle, StaticSoundSettings,
};
pub use kira::{
    track::{
        effect::{
            filter::{FilterBuilder, FilterHandle},
            reverb::ReverbBuilder,
        },
        TrackBuilder, TrackHandle,
    },
    Volume,
};

pub fn constant(_t: f32) -> f32 {
    0.0
}

#[derive(Clone, Debug)]
pub struct Name {
    pub name: Cow<'static, str>,
    // TODO: maybe hide under a feature flag to optionally track entity creation location?
    // pub backtrace: Backtrace,
}

impl Name {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            // backtrace: Backtrace::new()
        }
    }
}

pub fn default_hash(value: &impl std::hash::Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub trait ComplexExt {
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl ComplexExt for Complex<f32> {
    fn lerp(self, other: Self, t: f32) -> Self {
        let real = self.re + t * (other.re - self.re);
        let imag = self.im + t * (other.im - self.im);
        Complex::new(real, imag)
    }
}

pub struct Damage(pub f32);

#[derive(Copy, Clone, Debug)]
pub struct ValueRange<T> {
    pub min: T,
    pub max: T,
    pub value: T,
    pub speed: T,
}

impl<T> ValueRange<T> {
    pub fn new(value: T, min: T, max: T, speed: T) -> Self {
        Self { min, max, value, speed }
    }
}

pub trait OptionExtensions {
    fn log_none(self, message: impl Into<Cow<'static, str>>) -> Self;
    fn log_none_f(self, f: impl FnOnce()) -> Self;
}

impl<T> OptionExtensions for Option<T> {
    fn log_none(self, message: impl Into<Cow<'static, str>>) -> Self {
        if self.is_none() {
            error!("{}", message.into());
        }

        self
    }

    fn log_none_f(self, f: impl FnOnce()) -> Self {
        if self.is_none() {
            f();
        }

        self
    }
}

pub struct GlobalParams {
    pub floats: HashMap<&'static str, ValueRange<f32>>,
    pub ints: HashMap<&'static str, ValueRange<i32>>,
    pub flags: HashMap<String, bool>,
}

impl GlobalParams {
    pub fn new() -> Self {
        let mut floats = HashMap::default();

        floats.insert(
            "filter-cutoff",
            ValueRange::new(100.0, 0.0, 20000.0, 10.0),
        );
        floats.insert("filter-resonance", ValueRange::new(0.0, 0.0, 1.0, 0.01));

        floats.insert("colorScale", ValueRange::new(1.0, 0.001, 20.0, 0.01));
        floats
            .insert("bloomThreshold", ValueRange::new(1.0, 0.001, 50.0, 0.05));
        floats.insert("bloom-lerp", ValueRange::new(0.5, 0.0, 1.0, 0.005));
        floats.insert("exposure", ValueRange::new(1.0, 0.001, 10.0, 0.01));
        floats.insert("bloomGamma", ValueRange::new(0.8, 0.001, 3.0, 0.01));

        floats.insert(
            "chromatic_aberration",
            ValueRange::new(0.0, 0.0, 50.0, 0.1),
        );

        floats.insert("contrast", ValueRange::new(1.0, 0.00, 10.0, 0.01));
        floats.insert("brightness", ValueRange::new(0.0, 0.00, 10.0, 0.01));
        floats.insert("saturation", ValueRange::new(1.0, 0.00, 10.0, 0.01));
        floats.insert("gamma", ValueRange::new(1.0, 0.001, 10.0, 0.01));
        floats.insert("shake_amount", ValueRange::new(0.0, 0.0, 10.0, 0.01));

        let mut ints = HashMap::default();

        ints.insert("bloom_alg", ValueRange::new(1, 0, 2, 1));
        ints.insert("physics_substeps", ValueRange::new(8, 1, 64, 1));
        ints.insert("tonemapping_alg", ValueRange::new(3, 0, 4, 1));

        let mut flags = HashMap::default();

        flags.insert("additive-blending".to_string(), true);

        Self { floats, ints, flags }
    }

    pub fn set(name: &'static str, value: f32) {
        GLOBAL_PARAMS
            .borrow_mut()
            .floats
            .entry(name)
            .and_modify(|e| e.value = value);
    }

    pub fn get(name: &str) -> f32 {
        GLOBAL_PARAMS
            .borrow()
            .floats
            .get(name)
            .cloned()
            .unwrap_or_else(|| {
                error!("Missing param {name}");
                ValueRange::new(0.0, 0.0, 0.0, 0.1)
            })
            .value
    }

    pub fn set_int(name: &'static str, value: i32) {
        GLOBAL_PARAMS
            .borrow_mut()
            .ints
            .entry(name)
            .and_modify(|e| e.value = value);
    }

    pub fn get_int(name: &str) -> i32 {
        GLOBAL_PARAMS
            .borrow()
            .ints
            .get(name)
            .cloned()
            .unwrap_or_else(|| {
                error!("Missing param {name}");
                ValueRange::new(0, 0, 0, 0)
            })
            .value
    }

    pub fn flag(name: &str) -> bool {
        *GLOBAL_PARAMS.borrow().flags.get(name).unwrap_or(&false)
    }

    pub fn toggle_flag(name: &str) {
        let flags = &mut GLOBAL_PARAMS.borrow_mut().flags;

        let entry = flags.entry(name.to_string()).or_insert(false);

        *entry = !*entry;
    }

    pub fn flag_set(name: &str, value: bool) {
        GLOBAL_PARAMS.borrow_mut().flags.insert(name.to_string(), value);
    }
}

// pub trait EntityExtensions {
//     fn u128(&self) -> u128;
//     fn has<T: Send + Sync + 'static>(&self, world: &World) -> bool;
// }
//
// impl EntityExtensions for Entity {
//     fn u128(&self) -> u128 {
//         self.to_bits().get() as u128
//     }
//
//     fn has<T: Send + Sync + 'static>(&self, world: &World) -> bool {
//         world.get::<&T>(*self).is_ok()
//     }
// }

pub trait ErrorHandlingExtensions {
    fn log_err(&self);
}

impl<T> ErrorHandlingExtensions for Option<T> {
    fn log_err(&self) {
        if self.is_none() {
            error!("Unexpected None");
        }
    }
}

impl<T, E> ErrorHandlingExtensions for std::result::Result<T, E>
where E: std::fmt::Debug
{
    fn log_err(&self) {
        if let Err(err) = self {
            error!("Unexpected {err:?}");
        }
    }
}

pub trait ResultExtensions<T> {
    fn log_err_ok(self) -> Option<T>;
}

impl<T, E> ResultExtensions<T> for std::result::Result<T, E>
where E: std::fmt::Debug
{
    fn log_err_ok(self) -> Option<T> {
        match self {
            Ok(val) => Some(val),
            Err(err) => {
                error!("Unexpected {err:?}");
                None
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FollowPlayer;

#[derive(Copy, Clone, Debug)]
pub struct PlayerTag;

pub fn rect_contains(center: Vec2, size: Vec2, point: Vec2) -> bool {
    let hx = size.x / 2.0;
    let hy = size.y / 2.0;

    point.x >= center.x - hx &&
        point.x <= center.x + hx &&
        point.y >= center.y - hy &&
        point.y <= center.y + hy
}

#[derive(Copy, Clone, Debug)]
pub struct IRect {
    pub offset: IVec2,
    pub size: IVec2,
}

impl IRect {
    pub fn new(offset: IVec2, size: IVec2) -> Self {
        IRect { offset, size }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub center: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(center: Vec2, size: Vec2) -> Self {
        Self { center, size }
    }

    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self { center: (min + max) / 2.0, size: max - min }
    }

    pub fn top_left(&self) -> Vec2 {
        self.center - self.size / 2.0
    }
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct FrameDataUniform {
    pub projection: [f32; 16],
    pub mouse_world: [f32; 2],
    pub mouse_screen: [f32; 2],
    pub time: f32,
    pub delta: f32,
    pub frame: i32,
    pub fps: f32,
    pub aspect_ratio: f32,
    pub _padding: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct FrameParams {
    pub frame: u32,
    pub delta: f32,
    pub time: f32,
}

pub struct SpriteDraw {
    pub texture: TextureHandle,
    pub position: Vec2,
    pub color: Color,
    pub z_index: i32,
    pub raw_draw: RawDrawParams,
}

pub type TextureLoadQueue = Vec<LoadedImage>;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd)]
pub enum BlendMode {
    #[default]
    None,
    // TODO: Rename to Add
    Additive,
    Alpha,
}

// TODO: ... get rid of this
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureParams {
    pub blend_mode: BlendMode,
}

#[derive(Clone, Debug)]
pub struct MeshDraw {
    pub mesh: Mesh,
    pub texture_params: TextureParams,
    pub shader: Option<ShaderInstance>,
    pub render_target: Option<RenderTargetId>,
}

pub struct DrawParams<'a> {
    pub aspect_ratio: f32,
    pub projection: Mat4,
    pub white_px: TextureHandle,

    pub clear_color: Color,
    pub lights: Vec<Light>,

    pub config: &'a mut GameConfig,

    pub frame: FrameParams,
    pub mesh_queue: Vec<MeshDraw>,

    pub particle_queue: Vec<ParticleDraw>,
    pub egui: &'a egui::Context,
}

#[derive(Copy, Clone, Debug)]
pub struct ParticleDraw {
    pub position: Vec3,
    pub rotation: f32,
    pub texture: TextureHandle,
    pub color: Color,
    pub size: Vec2,
    pub source_rect: Option<IRect>,
    pub blend_mode: BlendMode,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct RawDrawParams {
    pub dest_size: Option<Vec2>,
    pub source_rect: Option<IRect>,
    pub rotation: f32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub pivot: Option<Vec2>,
}

const WHITE_ARRAY: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

pub const QUAD_VERTICES: &[SpriteVertex] = &[
    SpriteVertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
        color: WHITE_ARRAY,
    },
    SpriteVertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
        color: WHITE_ARRAY,
    },
    SpriteVertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
        color: WHITE_ARRAY,
    },
    SpriteVertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
        color: WHITE_ARRAY,
    },
];

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub vertices: SmallVec<[SpriteVertex; 4]>,
    pub indices: SmallVec<[u32; 6]>,
    pub z_index: i32,
    pub texture: Option<TextureHandle>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

impl SpriteVertex {
    pub fn new(position: Vec3, tex_coords: Vec2, color: Color) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            tex_coords: [tex_coords.x, tex_coords.y],
            color: [color.r, color.g, color.b, color.a],
        }
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// #[cfg(feature = "tracy")]
// type SpanType = tracy_client::Span;
// #[cfg(not(feature = "tracy"))]
// type SpanType = ();
//
// pub fn span(
//     #[cfg(feature = "tracy")] name: &str,
//     #[cfg(not(feature = "tracy"))] _name: &str,
// ) -> Option<SpanType> {
//     cfg_if::cfg_if! {
//         if #[cfg(feature = "tracy")] {
//             Some(tracy_client::span!(name, 0))
//         } else {
//             None
//         }
//     }
// }

// None::<()>
// cfg_if::cfg_if! {
//         if #[cfg(feature = "tracy")] {
//         Some(tracy_client::span!($name, 0))
//     } else {
//         None
//     }
// }

#[macro_export]
macro_rules! span_with_timing {
    ($name: expr) => {
        let (_s1, _s2) = (span!($name), timing_start($name));
    };
}

#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! span {
    ($name: expr) => {
        Some(tracy_client::span!($name, 0))
    };
}

#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! span {
    ($name: expr) => {
        None::<()>
    };
}

#[derive(
    Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, PartialEq,
)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn gray(value: f32) -> Self {
        Self { r: value, g: value, b: value, a: 1.0 }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn egui(self) -> egui::Color32 {
        self.into()
    }

    pub fn to_vec4(&self) -> Vec4 {
        Vec4::new(self.r, self.g, self.b, self.a)
    }

    pub fn to_array(self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }

    pub fn alpha(&self, value: f32) -> Color {
        Color::new(self.r, self.g, self.b, value)
    }

    pub fn mix(&self, other: Color, value: f32) -> Color {
        let a = 1.0 - value;
        let b = value;

        Color::new(
            self.r * a + other.r * b,
            self.g * a + other.g * b,
            self.b * a + other.b * b,
            self.a * a + other.a * b,
        )
    }

    pub fn to_image_rgba(self) -> image::Rgba<u8> {
        image::Rgba([
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ])
    }


    pub fn darken(&self, amount: f32) -> Color {
        let amount = 1.0 - amount;
        Color::new(self.r * amount, self.g * amount, self.b * amount, self.a)
    }

    // pub fn lighten(&self, amount: f32) -> Color {
    //     let amount = 1.0 - amount;
    //     Color::new(self.r * amount, self.g * amount, self.b * amount, self.a)
    // }

    pub fn lighten(&self, amount: f32) -> Color {
        let r = (self.r + amount).min(1.0);
        let g = (self.g + amount).min(1.0);
        let b = (self.b + amount).min(1.0);
        Color::new(r, g, b, self.a)
    }

    pub fn boost(&self, amount: f32) -> Color {
        Color::new(self.r * amount, self.g * amount, self.b * amount, self.a)
    }
}

impl From<Color> for image::Rgba<u8> {
    fn from(value: Color) -> Self {
        image::Rgba(value.to_array())
    }
}

impl From<image::Rgba<u8>> for Color {
    fn from(value: image::Rgba<u8>) -> Self {
        Self::rgba8(value.0[0], value.0[1], value.0[2], value.0[3])
    }
}

impl From<Color> for egui::Color32 {
    fn from(value: Color) -> Self {
        egui::Color32::from_rgba_unmultiplied(
            (value.r * 255.0) as u8,
            (value.g * 255.0) as u8,
            (value.b * 255.0) as u8,
            (value.a * 255.0) as u8,
        )
    }
}

impl Add<Color> for Color {
    type Output = Color;

    fn add(self, val: Color) -> Self::Output {
        Color::new(
            self.r + val.r,
            self.g + val.g,
            self.b + val.b,
            (self.a + val.a).clamp(0.0, 1.0),
        )
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, val: Color) -> Self::Output {
        Color::new(
            self.r * val.r,
            self.g * val.g,
            self.b * val.b,
            self.a * val.a,
        )
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, val: f32) -> Self::Output {
        Color::new(self.r * val, self.g * val, self.b * val, self.a)
    }
}

// impl Into<egui::Color32> for Color {
//     fn into(self) -> egui::Color32 {
//         egui::Color32::from_rgba_unmultiplied(
//             (self.r * 255.0) as u8,
//             (self.g * 255.0) as u8,
//             (self.b * 255.0) as u8,
//             (self.a * 255.0) as u8,
//         )
//     }
// }

pub fn font_family(name: &str, size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name(name.into()))
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Sound {
    pub id: u64,
}

impl Sound {
    pub fn from_path(path: &str) -> Sound {
        Sound { id: simple_hash(path) }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TextureHandle {
    Path(u64),
    Raw(u64),
    RenderTarget(RenderTargetId),
}

pub fn simple_hash(value: impl std::hash::Hash) -> u64 {
    ahash::RandomState::with_seeds(1, 2, 3, 4).hash_one(value)
}

impl TextureHandle {
    // TODO: rename to something like "unchecked_id"
    pub fn from_path(path: &str) -> Self {
        TextureHandle::Path(simple_hash(path))
    }

    pub fn key_unchecked(key: &str) -> Self {
        TextureHandle::Path(simple_hash(key))
    }
}

pub const LIGHTGRAY: Color = Color::new(0.78, 0.78, 0.78, 1.00);
pub const GRAY: Color = Color::new(0.51, 0.51, 0.51, 1.00);
pub const DARKGRAY: Color = Color::new(0.31, 0.31, 0.31, 1.00);
pub const YELLOW: Color = Color::new(0.99, 0.98, 0.00, 1.00);
pub const GOLD: Color = Color::new(1.00, 0.80, 0.00, 1.00);
pub const ORANGE: Color = Color::new(1.00, 0.63, 0.00, 1.00);
pub const PINK: Color = Color::new(1.00, 0.43, 0.76, 1.00);
pub const RED: Color = Color::new(0.90, 0.16, 0.22, 1.00);
pub const MAROON: Color = Color::new(0.75, 0.13, 0.22, 1.00);
pub const GREEN: Color = Color::new(0.00, 0.89, 0.19, 1.00);
pub const LIME: Color = Color::new(0.00, 0.62, 0.18, 1.00);
pub const DARKGREEN: Color = Color::new(0.00, 0.46, 0.17, 1.00);
pub const SKYBLUE: Color = Color::new(0.40, 0.75, 1.00, 1.00);
pub const BLUE: Color = Color::new(0.00, 0.47, 0.95, 1.00);
pub const DARKBLUE: Color = Color::new(0.00, 0.32, 0.67, 1.00);
pub const PURPLE: Color = Color::new(0.78, 0.48, 1.00, 1.00);
pub const VIOLET: Color = Color::new(0.53, 0.24, 0.75, 1.00);
pub const DARKPURPLE: Color = Color::new(0.44, 0.12, 0.49, 1.00);
pub const BEIGE: Color = Color::new(0.83, 0.69, 0.51, 1.00);
pub const BROWN: Color = Color::new(0.50, 0.42, 0.31, 1.00);
pub const DARKBROWN: Color = Color::new(0.30, 0.25, 0.18, 1.00);
pub const WHITE: Color = Color::new(1.00, 1.00, 1.00, 1.00);
pub const BLACK: Color = Color::new(0.00, 0.00, 0.00, 1.00);
pub const BLANK: Color = Color::new(0.00, 0.00, 0.00, 0.00);
pub const MAGENTA: Color = Color::new(1.00, 0.00, 1.00, 1.00);
pub const DARKRED: Color = Color::new(0.46, 0.08, 0.12, 1.00);
pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);
pub const ALICE_BLUE: Color = Color::rgb(0.94, 0.97, 1.0);
pub const ANTIQUE_WHITE: Color = Color::rgb(0.98, 0.92, 0.84);
pub const AQUAMARINE: Color = Color::rgb(0.49, 1.0, 0.83);
pub const AZURE: Color = Color::rgb(0.94, 1.0, 1.0);
pub const BISQUE: Color = Color::rgb(1.0, 0.89, 0.77);
pub const CRIMSON: Color = Color::rgb(0.86, 0.08, 0.24);
pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
pub const DARK_GRAY: Color = Color::rgb(0.25, 0.25, 0.25);
pub const DARK_GREEN: Color = Color::rgb(0.0, 0.5, 0.0);
pub const FUCHSIA: Color = Color::rgb(1.0, 0.0, 1.0);
pub const INDIGO: Color = Color::rgb(0.29, 0.0, 0.51);
pub const LIME_GREEN: Color = Color::rgb(0.2, 0.8, 0.2);
pub const MIDNIGHT_BLUE: Color = Color::rgb(0.1, 0.1, 0.44);
pub const NAVY: Color = Color::rgb(0.0, 0.0, 0.5);
pub const OLIVE: Color = Color::rgb(0.5, 0.5, 0.0);
pub const ORANGE_RED: Color = Color::rgb(1.0, 0.27, 0.0);
pub const SALMON: Color = Color::rgb(0.98, 0.5, 0.45);
pub const SEA_GREEN: Color = Color::rgb(0.18, 0.55, 0.34);
pub const SILVER: Color = Color::rgb(0.75, 0.75, 0.75);
pub const TEAL: Color = Color::rgb(0.0, 0.5, 0.5);
pub const TOMATO: Color = Color::rgb(1.0, 0.39, 0.28);
pub const TURQUOISE: Color = Color::rgb(0.25, 0.88, 0.82);
pub const YELLOW_GREEN: Color = Color::rgb(0.6, 0.8, 0.2);

// Comfy colors
// pub const COMFY_BLUE: Color = Color::rgb(0.16, 0.04, 0.02);
// pub const COMFY_BLUE: Color = Color::rgb(0.84, 0.96, 0.98);
pub const COMFY_BLUE: Color = Color::rgb(0.74, 0.86, 0.88);
pub const COMFY_PINK: Color = Color::rgb(1.0, 0.76, 0.86);
pub const COMFY_GREEN: Color = Color::rgb(0.67, 0.92, 0.72);
pub const COMFY_DARK_BLUE: Color = Color::rgb(0.73, 0.93, 0.97);

// pub const PINK: Color = Color::rgb(1.0, 0.08, 0.58);
// pub const PURPLE: Color = Color::rgb(0.5, 0.0, 0.5);
// pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
// pub const VIOLET: Color = Color::rgb(0.93, 0.51, 0.93);
// pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
// pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
// pub const BEIGE: Color = Color::rgb(0.96, 0.96, 0.86);
// pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
// pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
// pub const GOLD: Color = Color::rgb(1.0, 0.84, 0.0);
// pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);
// pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
// pub const MAROON: Color = Color::rgb(0.5, 0.0, 0.0);
// pub const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
// pub const ORANGE: Color = Color::rgb(1.0, 0.65, 0.0);

pub trait UVec2Extensions {
    fn fit_width(self, width: u32) -> egui::Vec2;
    fn fit_height(self, height: u32) -> egui::Vec2;
    fn fit_rect(self, width: u32, height: u32) -> egui::Vec2;
    fn fit_square(self, size: u32) -> egui::Vec2;
}

impl UVec2Extensions for UVec2 {
    fn fit_width(self, width: u32) -> egui::Vec2 {
        let ratio = self.y as f32 / self.x as f32;
        egui::vec2(width as f32, width as f32 * ratio)
    }

    fn fit_height(self, height: u32) -> egui::Vec2 {
        let ratio = self.x as f32 / self.y as f32;
        egui::vec2(height as f32 * ratio, height as f32)
    }

    fn fit_square(self, size: u32) -> egui::Vec2 {
        self.fit_rect(size, size)
    }

    fn fit_rect(self, width: u32, height: u32) -> egui::Vec2 {
        let size = vec2(width as f32, height as f32);
        let self_ratio = self.x as f32 / self.y as f32;
        let rect_ratio = size.x / size.y;

        if self_ratio > rect_ratio {
            // Image is wider than the rect, so fit to the rect's width
            self.fit_width(size.x as u32)
        } else {
            // Image is taller than the rect, so fit to the rect's height
            self.fit_height(size.y as u32)
        }
    }
}

pub trait Vec2Extensions {
    fn normalize_or_right(self) -> Vec2;
    fn tuple(self) -> (f32, f32);
    fn wiggle(self, angle: f32) -> Vec2;
    fn angle(self) -> f32;

    fn as_array(&self) -> [f32; 2];
    fn as_transform(&self) -> Transform;
    fn egui(&self) -> egui::Vec2;
    fn egui_pos(&self) -> egui::Pos2;
}

impl Vec2Extensions for Vec2 {
    fn normalize_or_right(self) -> Vec2 {
        let rcp = self.length_recip();

        if rcp.is_finite() && rcp > 0.0 {
            self * rcp
        } else {
            Self::X
        }
    }

    fn tuple(self) -> (f32, f32) {
        (self.x, self.y)
    }

    fn wiggle(self, angle: f32) -> Vec2 {
        self.rotate(Vec2::from_angle(gen_range(-angle / 2.0, angle / 2.0)))
    }

    fn angle(self) -> f32 {
        vec2(1.0, 0.0).angle_between(self)
    }

    fn as_array(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    fn as_transform(&self) -> Transform {
        Transform::position(*self)
    }

    fn egui(&self) -> egui::Vec2 {
        egui::vec2(self.x, self.y)
    }

    fn egui_pos(&self) -> egui::Pos2 {
        egui::pos2(self.x, self.y)
    }
}

// pub trait ColorExtensions {
//     fn with_alpha(&self, alpha: f32) -> Self;
//     fn to_egui(&self) -> egui::Color32;
//     fn is_similar(&self, other: Color, tolerance: f32) -> bool;
//     fn is_similar_no_alpha(&self, other: Color, tolerance: f32) -> bool;
//     fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self;
// }


#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SemanticVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl std::fmt::Display for SemanticVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[macro_export]
macro_rules! define_versions {
    () => {
        #[cfg(feature = "git-version")]
        pub const GIT_VERSION: &str = git_version::git_version!();

        $crate::lazy_static! {
            pub static ref VERSION: $crate::SemanticVer = $crate::SemanticVer {
                major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
                minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
                patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
            };
        }

        #[cfg(not(feature = "git-version"))]
        pub fn version_str() -> &'static str {
            concat!(
                "v",
                env!("CARGO_PKG_VERSION_MAJOR"),
                ".",
                env!("CARGO_PKG_VERSION_MINOR"),
                ".",
                env!("CARGO_PKG_VERSION_PATCH"),
            )
        }

        #[cfg(feature = "git-version")]
        pub fn version_str() -> &'static str {
            concat!(
                "v",
                env!("CARGO_PKG_VERSION_MAJOR"),
                ".",
                env!("CARGO_PKG_VERSION_MINOR"),
                ".",
                env!("CARGO_PKG_VERSION_PATCH"),
                " (",
                git_version::git_version!(),
                ")"
            )
        }
    };
}

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: f32,
    pub parent: Option<Entity>,

    pub abs_position: Vec2,
    pub abs_rotation: f32,
    pub abs_scale: f32,
}

impl Transform {
    pub fn position(position: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: 1.0,

            parent: None,

            abs_position: position,
            abs_rotation: 0.0,
            abs_scale: 1.0,
        }
    }

    pub fn rotation(self, rotation: f32) -> Self {
        Self { rotation, ..self }
    }

    pub fn scale(self, scale: f32) -> Self {
        Self { scale, ..self }
    }

    pub fn distance(&self, other: &Transform) -> f32 {
        self.position.distance(other.position)
    }

    pub fn parent(self, parent: Entity) -> Self {
        Self { parent: Some(parent), ..self }
    }


    pub fn compose_with_parent(
        &self,
        parent_transform: &Transform,
    ) -> Transform {
        let parent_matrix = parent_transform.to_matrix();
        let self_matrix = self.to_matrix();
        let composed_matrix = parent_matrix * self_matrix;

        let composed_transform = Transform::from_matrix(composed_matrix);

        Transform {
            position: composed_transform.position,
            rotation: composed_transform.rotation,
            // rotation: self.rotation,
            scale: composed_transform.scale,

            parent: None,

            abs_position: composed_transform.position,
            abs_rotation: composed_transform.rotation,
            abs_scale: composed_transform.scale,
        }
    }

    pub fn to_matrix(&self) -> Mat3 {
        let translate_matrix = Mat3::from_translation(self.position);
        let rotate_matrix = Mat3::from_angle(self.rotation);
        let scale_matrix = Mat3::from_scale(splat(self.scale));

        translate_matrix * rotate_matrix * scale_matrix
    }

    pub fn from_matrix(matrix: Mat3) -> Self {
        let position = matrix.transform_point2(Vec2::ZERO);
        // let rotation = matrix.x_axis.angle_between(Vec3::X);
        let rotation = f32::atan2(matrix.x_axis.y, matrix.x_axis.x);
        let scale = matrix.x_axis.length();

        Transform {
            position,
            rotation,
            scale,

            parent: None,

            abs_position: position,
            abs_rotation: rotation,
            abs_scale: scale,
        }
    }
}

pub fn initialize_logger() {
    #[cfg(feature = "file_logger")]
    {
        pub fn initialize_log4rs(
            log_root: &std::path::Path,
        ) -> Result<(), Box<dyn std::error::Error>> {
            use chrono::Timelike;
            use log4rs::{append::file::*, config::*, Config};

            let now = chrono::Utc::now();
            let (is_pm, hour) = now.hour12();

            let log_file = format!(
                "{} {}-{}-{} {}.log",
                now.date_naive(),
                hour,
                now.minute(),
                now.second(),
                if is_pm { "pm" } else { "am" }
            );

            let log_location = log_root.join(log_file);

            let logfile = FileAppender::builder().build(&log_location)?;
            let config = Config::builder()
                .appender(
                    Appender::builder().build("logfile", Box::new(logfile)),
                )
                .build(
                    Root::builder()
                        .appender("logfile")
                        .build(log::LevelFilter::Info),
                )?;

            log4rs::init_config(config)?;

            // Clear the log
            std::fs::write(&log_location, "")?;

            Ok(())
        }

        initialize_log4rs(std::path::Path::new("logs")).unwrap_or_else(|err| {
            eprintln!("FAILED TO INITIALIZE LOG4RS: {}", err);
        });

        println!("LOGGER: log4rs ");
    }

    #[cfg(not(feature = "file_logger"))]
    {
        env_logger::builder().format_timestamp(None).init();
        // env_logger::builder().format_timestamp_millis().init();

        println!("LOGGER: env_logger");
    }
}

pub trait MathExtensions {
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl MathExtensions for f32 {
    fn lerp(self, other: Self, t: f32) -> Self {
        self * (1.0 - t) + other * t
    }
}

impl MathExtensions for Color {
    fn lerp(self, other: Self, t: f32) -> Self {
        Color {
            r: self.r.lerp(other.r, t),
            g: self.g.lerp(other.g, t),
            b: self.b.lerp(other.b, t),
            a: self.a.lerp(other.a, t),
        }
    }
}

// pub const fn from_hex(hex: u32) -> Color {
//     let bytes: [u8; 4] = hex.to_be_bytes();
//
//     color_u8!(bytes[1], bytes[2], bytes[3], 255)
// }

pub trait RangeExtensions {
    fn lerp(self, t: f32) -> f32;
}

impl RangeExtensions for Range<f32> {
    fn lerp(self, t: f32) -> f32 {
        self.start.lerp(self.end, t)
    }
}

#[macro_export]
macro_rules! hash {
    ($a:expr) => {{
        let mut hasher = DefaultHasher::new();
        $a.hash(&mut hasher);
        hasher.finish()
    }};
    ($a:expr, $b:expr) => {{
        let mut hasher = DefaultHasher::new();
        $a.hash(&mut hasher);
        $b.hash(&mut hasher);
        hasher.finish()
    }};
}

pub fn timed_frame(interval: f32, frames: u32) -> i32 {
    ((get_time() / interval as f64) % frames as f64) as i32
}

pub fn timed_frame_from(start: f64, interval: f32, frames: u32) -> i32 {
    let time = (get_time() - start).max(0.0);

    ((time / interval as f64) % frames as f64) as i32
}

pub fn random_timed_frame(seed: f32, interval: f32, frames: u32) -> i32 {
    let off = (seed as f64).exp().sin() + 1.0;

    (((off + get_time()) / interval as f64) % frames as f64) as i32
}

pub fn random_entity_idx(entity: Entity, max: i32) -> usize {
    (entity.id() as i32 % max) as usize
}

pub trait EntityExtensions {
    fn to_user_data(&self) -> u128;
}

impl EntityExtensions for Entity {
    fn to_user_data(&self) -> u128 {
        self.to_bits().get().into()
    }
}

#[macro_export]
macro_rules! define_asset_dir {
    ($name:literal) => {
        define_asset_dir!($name, "assets");
    };

    ($name:literal, $dir:literal) => {
        cfg_if! {
            if #[cfg(feature = "ci-release")] {
                pub static ASSET_DIR: include_dir::Dir<'_> =
                    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../" $dir "/" $name "/assets");
            } else {
                pub static ASSET_DIR: include_dir::Dir<'_> =
                    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../" $dir "/" $name "/assets");
            }
        }

        fn base_path(path: &str) -> String {
            if cfg!(feature = "ci-release") {
                path.to_string()
            } else {
                format!(concat!("../", $dir, "/", $name, "/assets/{}"), path)
            }
        }
    };
}

pub fn triangle_wave(value: f32) -> f32 {
    triangle_wave_period(value, 2.0)
}

pub fn triangle_time(offset: f32) -> f32 {
    triangle_wave(offset + get_time() as f32)
}

pub fn triangle_wave_period(value: f32, period: f32) -> f32 {
    let t = (value % period) / period;
    if t < 0.5 {
        2.0 * t
    } else {
        2.0 * (1.0 - t)
    }
}

pub struct MovingAverage {
    size: usize,
    queue: VecDeque<f32>,
    sum: f32,
}

impl MovingAverage {
    pub fn new(size: usize) -> Self {
        MovingAverage { size, queue: VecDeque::with_capacity(size), sum: 0.0 }
    }

    pub fn next(&mut self, val: f32) -> f32 {
        if self.queue.len() == self.size {
            self.sum -= self.queue.pop_front().unwrap();
        }

        self.queue.push_back(val);
        self.sum += val;

        self.sum / self.queue.len() as f32
    }
}

pub struct MovingStats {
    size: usize,
    queue: VecDeque<f32>,
    sum: f32,
    sq_sum: f32,
}

pub struct Stats {
    pub mean: f32,
    pub std_dev: f32,
    pub percentile_50: f32,
    pub percentile_75: f32,
    pub percentile_90: f32,
    pub percentile_95: f32,
    pub percentile_99: f32,
}

impl MovingStats {
    pub fn new(size: usize) -> Self {
        MovingStats {
            size,
            queue: VecDeque::with_capacity(size),
            sum: 0.0,
            sq_sum: 0.0,
        }
    }

    pub fn next(&mut self, val: f32) -> Stats {
        if self.queue.len() == self.size {
            let old_val = self.queue.pop_front().unwrap();
            self.sum -= old_val;
            self.sq_sum -= old_val * old_val;
        }

        self.queue.push_back(val);
        self.sum += val;
        self.sq_sum += val * val;

        let queue_len = self.queue.len() as f32;
        let mean = self.sum / queue_len;
        let variance = (self.sq_sum / queue_len) - (mean * mean);
        let std_dev = f32::sqrt(variance.max(0.0)); // Ensure that the variance is non-negative

        let mut sorted_window: Vec<_> = self.queue.iter().collect();
        sorted_window.sort_by(|a, b| b.partial_cmp(a).unwrap());

        let get_percentile = |p: f32| {
            let idx = (p * queue_len).round() as usize;
            sorted_window[idx.min(sorted_window.len() - 1)]
        };

        Stats {
            mean,
            std_dev,
            percentile_50: *get_percentile(0.5),
            percentile_75: *get_percentile(0.75),
            percentile_90: *get_percentile(0.9),
            percentile_95: *get_percentile(0.95),
            percentile_99: *get_percentile(0.99),
        }
    }
}


// pub struct MovingStats {
//     size: usize,
//     queue: VecDeque<f32>,
//     sum: f32,
//     sq_sum: f32,
// }
//
// impl MovingStats {
//     pub fn new(size: usize) -> Self {
//         MovingStats {
//             size,
//             queue: VecDeque::with_capacity(size),
//             sum: 0.0,
//             sq_sum: 0.0,
//         }
//     }
//
//     pub fn next(&mut self, val: f32) -> (f32, f32) {
//         if self.queue.len() == self.size {
//             let old_val = self.queue.pop_front().unwrap();
//             self.sum -= old_val;
//             self.sq_sum -= old_val * old_val;
//         }
//
//         self.queue.push_back(val);
//         self.sum += val;
//         self.sq_sum += val * val;
//
//         let mean = self.sum / self.queue.len() as f32;
//         let variance = (self.sq_sum / self.queue.len() as f32) - (mean * mean);
//         let std_dev = f32::sqrt(variance.max(0.0)); // Ensure that the variance is non-negative
//         (mean, std_dev)
//     }
// }

pub struct ExponentialMovingAverage {
    alpha: f32,
    value: Option<f32>,
}

impl ExponentialMovingAverage {
    pub fn new(alpha: f32) -> Self {
        ExponentialMovingAverage { alpha, value: None }
    }

    pub fn next(&mut self, val: f32) -> f32 {
        match self.value {
            Some(prev_val) => {
                let new_val = self.alpha * val + (1.0 - self.alpha) * prev_val;
                self.value = Some(new_val);
                new_val
            }
            None => {
                self.value = Some(val);
                val
            }
        }
    }
}

pub fn is_point_in_rotated_rect(
    point: Vec2,
    rect_center: Vec2,
    rect_size: Vec2,
    rect_rotation: f32,
) -> bool {
    // Create the transformation matrix
    let transform = Mat3::from_translation(rect_center) *
        Mat3::from_rotation_z(rect_rotation);

    // Invert the transformation matrix to apply the reverse transformations to the point
    let inv_transform = transform.inverse();

    // Apply the transformations
    let new_point = inv_transform.transform_point2(point);

    // Check if the point is within the rectangle
    (new_point.x.abs() <= rect_size.x / 2.0) &&
        (new_point.y.abs() <= rect_size.y / 2.0)
}

// pub fn rescale(
//     value: i32,
//     from: Range<i32>,
//     to: Range<i32>,
// ) -> f32 {
//     let value = value.max(from.start).min(from.end);
//     let from_range = from.end - from.start;
//     let to_range = to.end - to.start;
//
//     to.start as f32 +
//         (value - from.start) as f32 / from_range as f32 * to_range as f32
// }

pub fn rescale<T: NumCast>(value: T, from: Range<T>, to: Range<T>) -> f32 {
    let value: f32 = NumCast::from(value).unwrap_or(0.0);
    let from_start = NumCast::from(from.start).unwrap_or(0.0);
    let from_end = NumCast::from(from.end).unwrap_or(0.0);
    let to_start = NumCast::from(to.start).unwrap_or(0.0);
    let to_end = NumCast::from(to.end).unwrap_or(0.0);

    let value = value.max(from_start).min(from_end);
    let from_range = from_end - from_start;
    let to_range = to_end - to_start;

    to_start + (value - from_start) / from_range * to_range
}

#[derive(Copy, Clone, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn from_two_points(a: Vec2, b: Vec2) -> Self {
        Self { min: a.min(b), max: a.max(b) }
    }

    pub fn from_top_left(top_left: Vec2, size: Vec2) -> Self {
        Self::from_center_size(
            vec2(top_left.x + size.x / 2.0, top_left.y - size.y / 2.0),
            size,
        )
    }

    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size * 0.5;
        Self { min: center - half_size, max: center + half_size }
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn contains(&self, point: Vec2) -> bool {
        self.min.x <= point.x &&
            self.min.y <= point.y &&
            self.max.x >= point.x &&
            self.max.y >= point.y
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x &&
            self.max.x >= other.min.x &&
            self.min.y <= other.max.y &&
            self.max.y >= other.min.y
    }

    pub fn expand_to_include_point(&mut self, point: Vec2) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    pub fn expand_to_include_aabb(&mut self, other: &AABB) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    pub fn top_left(&self) -> Vec2 {
        vec2(self.min.x, self.max.y)
    }
}

pub trait VecExtensions {
    fn flip(&self, width: usize) -> Self;
    fn flip_inplace(&mut self, width: usize);
}

impl<T: Clone> VecExtensions for Vec<T> {
    fn flip(&self, width: usize) -> Self {
        let mut res = self.clone();
        res.flip_inplace(width);
        res
    }

    fn flip_inplace(&mut self, width: usize) {
        assert!(self.len() % width == 0);

        let height = self.len() / width;

        for y in 0..(height / 2) {
            for x in 0..width {
                self.swap(y * width + x, (height - y - 1) * width + x)
            }
        }
    }
}

#[test]
fn test_vec_flip_h() {
    assert_eq!(vec![0, 0, 1, 1].flip(2), vec![1, 1, 0, 0]);
    assert_eq!(vec![0, 0, 0, 1, 1, 2].flip(3), vec![1, 1, 2, 0, 0, 0]);
    assert_eq!(vec![0, 0, 0, 1, 1, 2].flip(2), vec![1, 2, 0, 1, 0, 0]);

    assert_eq!(vec![0, 0, 0, 1, 1, 2, 3, 3].flip(2), vec![
        3, 3, 1, 2, 0, 1, 0, 0
    ]);
    assert_eq!(vec![0, 0, 0, 1, 1, 2, 3, 3].flip(4), vec![
        1, 2, 3, 3, 0, 0, 0, 1
    ]);
}
