use comfy_core::*;
use macroquad::window::{screen_height, screen_width};
use std::sync::mpsc::{channel, Receiver, Sender};

pub use macroquad;
pub use egui_macroquad;

mod text;

pub use crate::text::*;

#[derive(Copy, Clone, Debug)]
pub enum AddressMode {
    ClampToEdge,
}

pub fn blood_canvas_reset() {}

pub type TextureMap = HashMap<TextureHandle, TextureHandle>;

pub fn load_texture_from_engine_bytes(
    name: &str,
    bytes: &[u8],
    textures: &mut TextureMap,
    address_mode: AddressMode,
) {
}

pub fn sprite_shader_from_fragment(source: &str) -> String {
    "".to_string()
    // format!("{}{}{}", CAMERA_BIND_GROUP_PREFIX, FRAG_SHADER_PREFIX, source)
}

pub fn watch_shader_path(
    path: &str,
    shader_id: ShaderId,
) -> notify::Result<()> {
    let path = Path::new(path).canonicalize().unwrap().to_path_buf();

    // let mut hot_reload = HOT_RELOAD.lock();
    // hot_reload.watch_path(path.as_path())?;
    // hot_reload.shader_paths.insert(path, shader_id);

    Ok(())
}

pub fn blood_canvas_update_and_draw(f: fn(IVec2, &CanvasBlock)) {}

#[derive(Debug)]
pub struct QuadTextureCreator {}

impl QuadTextureCreator {
    pub fn new() -> Self {
        Self {}
    }
}

impl TextureCreator for QuadTextureCreator {
    fn handle_from_size(
        &self,
        name: &str,
        size: UVec2,
        fill: Color,
    ) -> TextureHandle {
        // TODO:
        TextureHandle::Raw(0)
    }

    fn handle_from_image(
        &self,
        name: &str,
        image: &image::RgbaImage,
    ) -> TextureHandle {
        // TODO:
        TextureHandle::Raw(0)
    }

    fn update_texture(&self, image: &image::RgbaImage, texture: TextureHandle) {
        // TODO:
    }

    fn update_texture_region(
        &self,
        handle: TextureHandle,
        image: &image::RgbaImage,
        region: IRect,
    ) {
        // TODO:
    }
}

pub static BLOOD_CANVAS: OnceCell<AtomicRefCell<BloodCanvas>> = OnceCell::new();

#[derive(Clone)]
pub struct GraphicsContext {
    pub texture_creator: Arc<AtomicRefCell<QuadTextureCreator>>,
}

pub struct QuadRenderer {
    pub context: GraphicsContext,
    pub texture_creator: Arc<AtomicRefCell<QuadTextureCreator>>,
    pub loaded_image_recv: Receiver<LoadedImage>,
    pub loaded_image_send: Sender<LoadedImage>,
    pub text: RefCell<TextRasterizer>,
    pub screenshot_params: ScreenshotParams,
    pub screenshot_history_buffer: VecDeque<ScreenshotItem>,
}

impl QuadRenderer {
    pub async fn new() -> Self {
        let texture_creator =
            Arc::new(AtomicRefCell::new(QuadTextureCreator::new()));

        let (tx_texture, rx_texture) = channel::<LoadedImage>();

        let context =
            GraphicsContext { texture_creator: texture_creator.clone() };

        QuadRenderer {
            text: RefCell::new(TextRasterizer::new(context.clone())),
            context,
            texture_creator,
            loaded_image_recv: rx_texture,
            loaded_image_send: tx_texture,
            screenshot_params: ScreenshotParams::default(),
            screenshot_history_buffer: VecDeque::new(),
        }
    }

    pub fn width(&self) -> f32 {
        screen_width()
    }

    pub fn height(&self) -> f32 {
        screen_height()
    }

    pub fn update(&mut self, params: &mut DrawParams) {}

    pub fn draw(&mut self, params: DrawParams, egui: &egui::Context) {}

    pub fn end_frame(&mut self) {}
}

pub fn save_screenshots_to_folder(
    folder: &str,
    screenshot_history_buffer: &VecDeque<ScreenshotItem>,
) {
    todo!()
}
