#![doc = include_str!("../README.md")]
#[cfg(feature="gif_player")]
mod gif_player;
#[cfg(feature="sprite")]
mod sprite;
#[cfg(feature="sprite_animation")]
mod sprite_anim;

#[cfg(feature="gif_player")]
pub use gif_player::{GifPlayer, GifPlayerError};
#[cfg(feature="sprite")]
pub use sprite::{Sprite, SpriteError, SpriteButton};
#[cfg(feature="sprite_animation")]
pub use sprite_anim::{SpriteAnimator, SpriteAnimatorError};
