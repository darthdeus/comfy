#![allow(clippy::uninlined_format_args)]
#![allow(clippy::new_without_default)]

mod animated_sprite;
mod animation;
mod cached_image_loader;
mod combat_text;
mod comfy_hecs;
mod context;
mod cooldowns;
mod debug;
mod draw;
mod egui_utils;
mod engine;
mod game;
mod game_loop;
mod macros;
mod particles;
mod render;
mod shaders;
mod timer;
mod trail;
mod update_stages;

pub use crate::animated_sprite::*;
pub use crate::animation::*;
pub use crate::cached_image_loader::*;
pub use crate::combat_text::*;
pub use crate::comfy_hecs::*;
pub use crate::context::*;
pub use crate::cooldowns::*;
pub use crate::debug::*;
pub use crate::draw::*;
pub use crate::egui_utils::*;
pub use crate::engine::*;
pub use crate::game::*;
pub use crate::game_loop::*;
// pub use crate::macros::*;
pub use crate::particles::*;
pub use crate::render::*;
pub use crate::shaders::*;
pub use crate::timer::*;
pub use crate::trail::*;
pub use crate::update_stages::*;

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

pub use comfy_core::{self, Assets, *};

pub use comfy_quad;
pub use comfy_quad::*;

#[cfg(feature = "ldtk")]
pub use comfy_ldtk::*;

#[cfg(feature = "tracy")]
pub use tracy_client::{frame_mark, secondary_frame_mark};

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
    _c: &EngineContext,
    ui: &mut egui::Ui,
    texture: &str,
    params: ImageButtonParams,
) -> egui::Response {
    image_button_without_c(
        text,
        &mut cached_loader_mut(),
        egui(),
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

    painter.galley(
        image_rect.center() - galley.mesh_bounds.size() / 2.0,
        galley,
        egui::Color32::PLACEHOLDER,
    );

    response
}

#[cfg(not(feature = "tracy"))]
pub fn maybe_setup_tracy() -> i32 {
    info!("TRACING DISABLED");
    // We don't really care about the value, but if () is returned rustc complains about binding ()
    0
}

#[cfg(feature = "tracy")]
pub fn maybe_setup_tracy() -> tracy_client::Client {
    info!("CONNECTING TO TRACY");

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

    tracy_client::Client::start()
}
