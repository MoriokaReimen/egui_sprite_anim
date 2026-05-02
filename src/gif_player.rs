use image::AnimationDecoder;
use std::collections::HashMap;
use std::io::Cursor;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur during the lifecycle of a [`GifPlayer`].
#[derive(Error, Debug)]
pub enum GifPlayerError {
    /// The specified key does not exist in the player's registry.
    #[error("Key {0} is not found")]
    KeyNotFound(String),

    /// An attempt was made to insert a key that is already present.
    #[error("The same key {0} is inserted already")]
    DuplicatedKey(String),

    /// The image data could not be parsed or decoded correctly.
    #[error("Failed to decode file")]
    DecodeFailed,

    /// The provided image source contains no valid frames to display.
    #[error("No Frame found in the image")]
    NoFrame,
}

/// A single frame of a GIF, stored as an `egui` texture with an associated display duration.
#[derive(Clone)]
pub struct GifFrame {
    /// The uploaded texture handle in egui.
    texture: egui::TextureHandle,
    /// How long this specific frame should be displayed.
    delay: Duration,
}

/// Internal state for a loaded GIF animation.
#[derive(Clone)]
struct Gif {
    /// Collection of all decoded frames.
    frames: Vec<GifFrame>,
    /// Index of the currently visible frame.
    current_frame_idx: usize,
    /// The timestamp of the last frame transition.
    last_update: Instant,
}

impl Gif {
    /// Creates a new GIF instance starting at the first frame.
    pub fn new(frames: Vec<GifFrame>) -> Self {
        Gif {
            frames,
            current_frame_idx: 0usize,
            last_update: Instant::now(),
        }
    }
}

/// A widget-ready manager for playing multiple GIF animations in an `egui` application.
/// 
/// `GifPlayer` acts as a registry for GIF files, allowing you to switch between 
/// different animations by key and control playback state.
pub struct GifPlayer {
    /// Storage for loaded animations, keyed by string identifiers.
    gifs: HashMap<String, Gif>,
    /// Optional override for the render size of the GIF.
    size: Option<egui::Vec2>,
    /// The key of the GIF currently selected for rendering.
    render_target: String,
    /// Whether the animation is currently progressing.
    is_playing: bool,
}

impl Default for GifPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl GifPlayer {
    /// Creates a new, empty `GifPlayer`.
    pub fn new() -> Self {
        Self {
            gifs: HashMap::new(),
            size: None,
            render_target: String::new(),
            is_playing: false,
        }
    }

    /// Decodes a GIF from bytes and adds it to the player's registry.
    ///
    /// # Arguments
    /// * `ctx` - The egui context used to upload textures.
    /// * `key` - A unique identifier for this GIF.
    /// * `gif_bytes` - The raw encoded data of the GIF file.
    ///
    /// # Errors
    /// Returns [`GifPlayerError::DuplicatedKey`] if the key already exists, 
    /// or decoding errors if the data is invalid.
    pub fn add(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        gif_bytes: &[u8],
    ) -> Result<(), GifPlayerError> {
        let hash = md5::compute(gif_bytes);
        let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(gif_bytes))
            .map_err(|_| GifPlayerError::DecodeFailed)?;
        let frames = decoder
            .into_frames()
            .collect_frames()
            .map_err(|_| GifPlayerError::NoFrame)?;

        let egui_frames = frames
            .into_iter()
            .enumerate()
            .map(|(i, frame)| {
                let delay = Duration::from(frame.delay());
                let buffer = frame.into_buffer();
                let size = [buffer.width() as usize, buffer.height() as usize];
                let pixels = buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                let texture = ctx.load_texture(
                    format!("{:x}-gif-frame-{}", hash, i),
                    color_image,
                    Default::default(),
                );

                GifFrame { texture, delay }
            })
            .collect();

        let render_target = key.to_string();
        if self.gifs.contains_key(key) {
            return Err(GifPlayerError::DuplicatedKey(render_target));
        }
        self.gifs.insert(key.to_string(), Gif::new(egui_frames));
        self.render_target = key.to_string();
        Ok(())
    }

    /// Sets the display size for the GIF. If `None`, it uses the GIF's native size.
    pub fn size(&mut self, size: impl Into<egui::Vec2>) {
        self.size = Some(size.into());
    }

    /// Switches the active animation to the one associated with the given `render_target`.
    ///
    /// # Errors
    /// Returns [`GifPlayerError::KeyNotFound`] if the key is not in the registry.
    pub fn select_image(&mut self, render_target: &str) -> Result<(), GifPlayerError> {
        let render_target = render_target.to_string();
        if !self.gifs.contains_key(&render_target) {
            Err(GifPlayerError::KeyNotFound(render_target))
        } else {
            self.render_target = render_target.to_string();
            Ok(())
        }
    }

    /// Resumes animation playback.
    pub fn play(&mut self) {
        self.is_playing = true;
    }

    /// Pauses animation playback at the current frame.
    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    /// Returns `true` if the animation is currently playing.
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
}

impl egui::Widget for &mut GifPlayer {
    /// Renders the currently selected GIF and manages frame timing for the next repaint.
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        assert!(self.gifs.contains_key(&self.render_target));
        let gif = self
            .gifs
            .get_mut(&self.render_target)
            .expect("Failed to get mutable Gif instance");
        let elapsed = Instant::now().duration_since(gif.last_update);
        let current_delay = gif.frames[gif.current_frame_idx].delay;

        if elapsed >= current_delay {
            if self.is_playing {
                gif.current_frame_idx = (gif.current_frame_idx + 1) % gif.frames.len();
            }
            gif.last_update = Instant::now();
        }
        
        // Ensure the UI refreshes when the next frame is due
        ui.ctx()
            .request_repaint_after(gif.frames[gif.current_frame_idx].delay);

        let texture = &gif.frames[gif.current_frame_idx].texture;
        let mut img = egui::Image::new(texture);
        if let Some(size) = self.size {
            img = img.fit_to_exact_size(size);
        }

        ui.add(img)
    }
}
