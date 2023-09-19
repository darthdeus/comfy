#![allow(clippy::uninlined_format_args)]
#![allow(clippy::new_without_default)]

mod animated_sprite;
mod animation;
mod context;
mod cooldowns;
mod debug;
mod draw;
mod egui_utils;
mod engine;
mod macros;
mod particles;
mod render;
mod shaders;
mod timer;
mod trail;

pub use crate::animated_sprite::*;
pub use crate::animation::*;
pub use crate::context::*;
pub use crate::cooldowns::*;
pub use crate::debug::*;
pub use crate::draw::*;
pub use crate::egui_utils::*;
pub use crate::engine::*;
pub use crate::macros::*;
pub use crate::particles::*;
pub use crate::render::*;
pub use crate::shaders::*;
pub use crate::timer::*;
pub use crate::trail::*;

pub use std::{
    any::Any,
    cell::RefCell,
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt,
    rc::Rc,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

pub use comfy_core;
pub use comfy_core::{Assets, *};

pub use std::path::Path;

pub use comfy_core::*;

pub use comfy_wgpu;
pub use comfy_wgpu::*;

#[cfg(feature = "tracy")]
pub use tracy_client::{frame_mark, secondary_frame_mark};

pub async fn run_comfy_main_async(game_state: Box<dyn RunGameLoop>) {
    if cfg!(feature = "tracy") {
        info!("CONNECTING TO TRACY");
    } else {
        info!("TRACING DISABLED");
    };

    #[cfg(feature = "tracy")]
    let _client = tracy_client::Client::start();

    // let file_appender = tracing_appender::rolling::daily("logs", "log"); //
    // This should be user configurable let (non_blocking, _worker_guard) =
    //     tracing_appender::non_blocking(file_appender);
    //
    // if cfg!(feature = "dev") {
    //     let subscriber = tracing_subscriber::FmtSubscriber::builder()
    //         .with_max_level(tracing::Level::INFO)
    //         .with_env_filter("wgpu=warn,symphonia=warn,game-lib=info,bod=info")
    //         .finish()
    //         .with(tracing_tracy::TracyLayer::default());
    //
    //     tracing::subscriber::set_global_default(subscriber).unwrap();
    // } else {
    //     // a builder for `FmtSubscriber`.
    //     let subscriber = tracing_subscriber::FmtSubscriber::builder()
    //         // all spans/events with a level higher than TRACE (e.g, debug,
    // info, warn, etc.)         // will be written to stdout.
    //         .with_max_level(tracing::Level::WARN)
    //         .with_ansi(false)
    //         .with_writer(non_blocking)
    //         .finish();
    //
    //     tracing::subscriber::set_global_default(subscriber)
    //         .expect("setting default subscriber failed");
    // }

    // let target_framerate = if cfg!(feature = "dev") { 10000 } else { 60 };
    // let target_framerate = if cfg!(feature = "dev") { 60 } else { 60 };

    #[cfg(not(target_arch = "wasm32"))]
    let target_framerate = 60;

    #[cfg(not(target_arch = "wasm32"))]
    let loop_helper = spin_sleep::LoopHelper::builder()
        .build_with_target_rate(target_framerate);

    // TODO: baaaaaaad, but for now ...
    #[cfg(target_arch = "wasm32")]
    let resolution = winit::dpi::PhysicalSize::new(960, 560);
    #[cfg(not(target_arch = "wasm32"))]
    let resolution = winit::dpi::PhysicalSize::new(1920, 1080);

    wgpu_game_loop(
        #[cfg(not(target_arch = "wasm32"))]
        loop_helper,
        game_state,
        resolution,
    )
    .await;
}

pub fn timed_two_frames(
    interval: f32,
    f1: &'static str,
    f2: &'static str,
) -> &'static str {
    if timed_frame(interval, 2) == 0 {
        f1
    } else {
        f2
    }
}

pub struct DespawnAfter(pub f32);

// TODO: remove
type DamagedCallback = fn(health_dmg: f32, shield_dmg: f32, c: &EngineContext);

// TODO: remove
pub struct Health {
    pub health_cur: f32,
    pub health_max: f32,
    pub health_regen: f32,

    pub shields_cur: f32,
    pub shields_max: f32,

    pub on_damaged: Option<DamagedCallback>,
}

impl Health {
    pub fn new(
        health_max: f32,
        shields_max: f32,
        // on_damaged: Option<DamagedCallback>,
    ) -> Self {
        Self {
            health_cur: health_max,
            health_max,
            health_regen: 0.0,

            shields_cur: shields_max,
            shields_max,

            on_damaged: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageButtonParams {
    pub font: egui::FontId,
    pub background_color: Color,
    pub wrap_width: f32,
    pub fixed_width: Option<f32>,
}

pub fn image_button(
    text: &str,
    wrap_width: f32,
    c: &EngineContext,
    ui: &mut egui::Ui,
    texture: &str,
) -> egui::Response {
    // let font = egui::FontId::proportional(16.0);
    let font = egui::FontId::new(16.0, egui::FontFamily::Proportional);

    image_button_ex(text, c, ui, texture, ImageButtonParams {
        font,
        background_color: WHITE,
        wrap_width,
        fixed_width: None,
    })
}

// TODO: remove
pub fn image_button_ex(
    text: &str,
    c: &EngineContext,
    ui: &mut egui::Ui,
    texture: &str,
    params: ImageButtonParams,
) -> egui::Response {
    image_button_without_c(
        text,
        &mut c.cached_loader.borrow_mut(),
        c.egui,
        ui,
        texture,
        params,
    )
}

pub fn image_button_without_c(
    text: &str,
    cached_loader: &mut CachedImageLoader,
    egui: &egui::Context,
    ui: &mut egui::Ui,
    texture: &str,
    params: ImageButtonParams,
) -> egui::Response {
    let text_color = WHITE.egui();

    let galley = ui.fonts(|f| {
        f.layout(text.to_string(), params.font, text_color, params.wrap_width)
    });

    let mut button_size = galley.mesh_bounds.size() + egui::vec2(40.0, 25.0);

    if let Some(width) = params.fixed_width {
        button_size.x = width;
    }

    let (response, painter) =
        ui.allocate_painter(button_size, egui::Sense::click());

    let rect = response.rect;

    let texture_id = cached_loader.image_or_err(egui, texture);

    let base_color = params.background_color;

    let color = if response.is_pointer_button_down_on() {
        base_color.darken(0.49)
    } else if response.hovered() {
        base_color.darken(0.22)
    } else {
        base_color
    };

    painter.image(
        texture_id,
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        color.egui(),
    );

    let image_rect = rect;

    painter
        .galley(image_rect.center() - galley.mesh_bounds.size() / 2.0, galley);

    response
}
