use egui::{Color32, ColorImage, Rect, TextureHandle, TextureOptions, Vec2, pos2};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur when managing or retrieving images from a [`Sprite`] sheet.
#[derive(Error, Debug)]
pub enum SpriteError {
    /// The image data provided could not be decoded into a valid pixel format.
    #[error("Failed to decode file")]
    DecodeFailed,

    /// The requested sub-rectangle is partially or fully outside the bounds of the base texture.
    #[error("Rect out of range")]
    InvalidRect,

    /// No mapping exists for the provided identifier.
    #[error("Rect ID {0} not found")]
    InvalidId(String),

    /// An attempt was made to add a mapping using a key that is already in use.
    #[error("The key {0} exists already")]
    DuplicatedId(String),
}

/// A manager for a sprite sheet texture and its associated sub-region mappings.
///
/// `Sprite` allows you to load a single large image (texture atlas) and define
/// named regions (rects) within it to be retrieved later as individual images or buttons.
#[derive(Clone)]
pub struct Sprite {
    /// The underlying texture handle uploaded to egui.
    texture: TextureHandle,
    /// A map of identifiers to their respective pixel coordinates on the texture.
    mapping: HashMap<String, Rect>,
}

impl Sprite {
    /// Creates a new [`Sprite`] instance by decoding image bytes and uploading them as a texture.
    ///
    /// # Arguments
    /// * `ctx` - The egui context required to allocate the texture.
    /// * `image_bytes` - The raw encoded bytes of the sprite sheet (e.g., PNG, JPEG).
    ///
    /// # Errors
    /// Returns [`SpriteError::DecodeFailed`] if the image bytes are invalid.
    pub fn new(ctx: &egui::Context, image_bytes: &[u8]) -> Result<Self, SpriteError> {
        let hash = md5::compute(image_bytes);
        let image = image::load_from_memory(image_bytes).map_err(|_| SpriteError::DecodeFailed)?;

        let size = [image.width() as usize, image.height() as usize];
        let color_image =
            ColorImage::from_rgba_unmultiplied(size, image.to_rgba8().as_flat_samples().as_slice());

        let texture = ctx.load_texture(format!("{:x}", hash), color_image, TextureOptions::NEAREST);

        Ok(Self {
            texture,
            mapping: HashMap::new(),
        })
    }

    /// Registers a named sub-region within the sprite sheet.
    ///
    /// # Arguments
    /// * `sprite_id` - A unique name for this sub-image.
    /// * `rect` - The pixel coordinates (min and max) defining the sprite region.
    ///
    /// # Errors
    /// * [`SpriteError::DuplicatedId`]: If the ID is already registered.
    /// * [`SpriteError::InvalidRect`]: If the rect exceeds the texture dimensions.
    pub fn add_map(&mut self, sprite_id: impl Into<String>, rect: Rect) -> Result<(), SpriteError> {
        let id = sprite_id.into();
        if self.mapping.contains_key(&id) {
            return Err(SpriteError::DuplicatedId(id));
        }

        let tex_rect = Rect::from_min_size(egui::Pos2::ZERO, self.texture.size_vec2());
        if !tex_rect.contains_rect(rect) {
            return Err(SpriteError::InvalidRect);
        }

        self.mapping.insert(id, rect);
        Ok(())
    }

    /// Internal helper to calculate UV coordinates and size for a given sprite ID.
    fn get_uv_and_size(&self, sprite_id: &str) -> Result<(Rect, Vec2), SpriteError> {
        let rect = self
            .mapping
            .get(sprite_id)
            .ok_or(SpriteError::InvalidId(sprite_id.to_string()))?;
        let tex_size = self.texture.size_vec2();

        let uv = Rect::from_min_max(
            pos2(rect.min.x / tex_size.x, rect.min.y / tex_size.y),
            pos2(rect.max.x / tex_size.x, rect.max.y / tex_size.y),
        );

        Ok((uv, rect.size()))
    }

    /// Returns an [`egui::Image`] representing the sprite associated with `sprite_id`.
    ///
    /// # Arguments
    /// * `sprite_id` - A unique name for this sub-image.
    ///
    /// # Errors
    /// Returns [`SpriteError::InvalidId`] if the ID does not exist.
    pub fn get(&self, sprite_id: &str) -> Result<egui::Image<'_>, SpriteError> {
        let (uv, size) = self.get_uv_and_size(sprite_id)?;

        Ok(egui::Image::new(&self.texture)
            .uv(uv)
            .fit_to_exact_size(size))
    }

    /// Returns a [`SpriteButton`] widget for the sprite associated with `sprite_id`.
    ///
    /// # Arguments
    /// * `sprite_id` - A unique name for this sub-image.
    ///
    /// # Errors
    /// Returns [`SpriteError::InvalidId`] if the ID does not exist.
    pub fn get_as_button(&self, sprite_id: &str) -> Result<SpriteButton<'_>, SpriteError> {
        let (uv, size) = self.get_uv_and_size(sprite_id)?;

        Ok(SpriteButton {
            texture: &self.texture,
            uv,
            size,
        })
    }
}

/// A clickable button widget that displays a specific sub-region of a sprite sheet.
///
/// Features built-in visual feedback (tinting) for hover and click states.
pub struct SpriteButton<'a> {
    texture: &'a TextureHandle,
    uv: Rect,
    size: Vec2,
}

impl egui::Widget for SpriteButton<'_> {
    /// Renders the sprite button and handles interaction logic.
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(self.size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let tint = if response.is_pointer_button_down_on() {
                Color32::from_gray(80) // Darken on click
            } else if response.hovered() {
                Color32::WHITE        // Full brightness on hover
            } else {
                Color32::from_gray(200) // Slightly dimmed by default
            };

            egui::Image::new(self.texture)
                .uv(self.uv)
                .tint(tint)
                .paint_at(ui, rect);
        }

        response
    }
}
