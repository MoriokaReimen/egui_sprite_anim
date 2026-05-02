use egui::TextureOptions;
use egui::{ColorImage, Image, Rect, Response, TextureHandle, Ui, pos2};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur during sprite animation management with [SpriteAnimator].
#[derive(Error, Debug)]
pub enum SpriteAnimatorError {
    /// The image data could not be parsed or decoded.
    #[error("Failed to decode file")]
    DecodeFailed,

    /// A specified frame rectangle is outside the bounds of the texture.
    #[error("Rect out of range")]
    InvalidRect,

    /// The requested animation identifier does not exist.
    #[error("Rect ID {0} not found")]
    InvalidId(String),

    /// An animation with the given identifier has already been registered.
    #[error("The key {0} exists already")]
    DuplicatedId(String),
}

/// Configuration for a specific animation sequence within a sprite sheet.
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// Sequence of frame definitions (pixel coordinates) on the texture.
    pub frames: Vec<Rect>,
    /// Playback speed in frames per second.
    pub fps: f32,
    /// Whether the animation should restart from the first frame upon completion.
    pub loop_anim: bool,
}

/// A stateful handler for playing sprite-sheet animations in `egui`.
/// 
/// `SpriteAnimator` manages timing, playback state (play/pause), and 
/// frame calculation for multiple animation sequences sharing the same texture.
#[derive(Clone)]
pub struct SpriteAnimator {
    /// The uploaded texture handle containing the sprite sheet.
    texture: TextureHandle,
    /// The ID of the currently active animation.
    render_target: String,
    /// Registered animation sequences.
    configs: HashMap<String, AnimationConfig>,
    /// The time when the current animation sequence (re)started.
    start_time: Instant,
    /// Playback status.
    is_playing: bool,
    /// The duration elapsed when the animation was last paused.
    paused_at: Duration,
}

impl SpriteAnimator {
    /// Creates a new [`SpriteAnimator`] by loading a sprite sheet from bytes.
    ///
    /// # Arguments
    /// * `ctx` - The egui context for texture allocation.
    /// * `image_bytes` - Raw encoded image data (PNG, JPG, etc.).
    ///
    /// # Errors
    /// Returns [`SpriteAnimatorError::DecodeFailed`] if image parsing fails.
    pub fn new(ctx: &egui::Context, image_bytes: &[u8]) -> Result<Self, SpriteAnimatorError> {
        let hash = md5::compute(image_bytes);
        let image =
            image::load_from_memory(image_bytes).map_err(|_| SpriteAnimatorError::DecodeFailed)?;

        let size = [image.width() as usize, image.height() as usize];
        let color_image =
            ColorImage::from_rgba_unmultiplied(size, image.to_rgba8().as_flat_samples().as_slice());

        let texture = ctx.load_texture(format!("{:x}", hash), color_image, TextureOptions::NEAREST);

        Ok(Self {
            texture,
            render_target: String::new(),
            configs: HashMap::new(),
            start_time: Instant::now(),
            is_playing: true,
            paused_at: Duration::ZERO,
        })
    }

    /// Calculates the current frame rectangle based on elapsed time and the animation config.
    ///
    /// # Errors
    /// Returns [`SpriteAnimatorError::InvalidId`] if the requested animation is not found.
    fn current_rect(&self, anim_id: &str) -> Result<Rect, SpriteAnimatorError> {
        let elapsed = if self.is_playing {
            self.start_time.elapsed()
        } else {
            self.paused_at
        };
        let config = self
            .configs
            .get(anim_id)
            .ok_or(SpriteAnimatorError::InvalidId(anim_id.to_string()))?;
        
        let total_frames = config.frames.len();
        if total_frames == 0 { return Err(SpriteAnimatorError::InvalidRect); }
        
        let frame_duration = 1.0 / config.fps;
        let mut frame_idx = (elapsed.as_secs_f32() / frame_duration) as usize;

        if config.loop_anim {
            frame_idx %= total_frames;
        } else {
            frame_idx = frame_idx.min(total_frames - 1);
        }

        Ok(config.frames[frame_idx])
    }

    /// Registers a new animation configuration.
    ///
    /// # Arguments
    /// * `anim_id` - Key linked to the animation configuration
    /// * `config` - animation configuration
    ///
    /// # Errors
    /// Returns [`SpriteAnimatorError::DuplicatedId`] if the ID is already taken.
    pub fn add_config(
        &mut self,
        anim_id: &str,
        config: &AnimationConfig,
    ) -> Result<(), SpriteAnimatorError> {
        if self.configs.contains_key(anim_id) {
            return Err(SpriteAnimatorError::DuplicatedId(anim_id.to_string()));
        }
        self.configs.insert(anim_id.to_string(), config.clone());
        self.render_target = anim_id.to_string();
        Ok(())
    }

    /// Switches the active animation to the sequence identified by `anim_id`.
    ///
    /// # Arguments
    /// * `anim_id` - Key linked to the animation configuration
    ///
    /// # Errors
    /// Returns [`SpriteAnimatorError::InvalidId`] if the ID does not exist.
    pub fn select_anim(&mut self, anim_id: &str) -> Result<(), SpriteAnimatorError> {
        if !self.configs.contains_key(anim_id) {
            return Err(SpriteAnimatorError::InvalidId(anim_id.to_string()));
        }
        self.render_target = anim_id.to_string();
        Ok(())
    }

    /// Resumes or starts animation playback.
    pub fn play(&mut self) {
        if !self.is_playing {
            self.start_time = Instant::now() - self.paused_at;
            self.is_playing = true;
        }
    }

    /// Pauses animation playback at the current frame.
    pub fn pause(&mut self) {
        if self.is_playing {
            self.paused_at = self.start_time.elapsed();
            self.is_playing = false;
        }
    }
}

impl egui::Widget for SpriteAnimator {
    /// Renders the current frame of the active animation and requests a repaint for the next frame.
    fn ui(self, ui: &mut Ui) -> Response {
        let config = self.configs.get(&self.render_target).expect("Active animation config missing");
        let rect = self.current_rect(&self.render_target).expect("Failed to calculate current frame");
        let tex_size = self.texture.size_vec2();

        // Convert pixel coordinates to UV normalized coordinates (0.0 to 1.0)
        let uv = Rect::from_min_max(
            pos2(rect.min.x / tex_size.x, rect.min.y / tex_size.y),
            pos2(rect.max.x / tex_size.x, rect.max.y / tex_size.y),
        );

        let display_size = rect.size();

        // Schedule the next UI update based on FPS
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f32(1.0 / config.fps));

        let img = Image::new(&self.texture)
            .uv(uv)
            .fit_to_exact_size(display_size);

        ui.add(img)
    }
}
