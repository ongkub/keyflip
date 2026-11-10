//! Portable KeyFlip n-gram scorer and layout decision engine.
pub mod detect;
pub mod keymap;
pub mod model;

pub use detect::{Config, Decision, Detector};
pub use keymap::{Keymap, Lang};
pub use model::Model;
