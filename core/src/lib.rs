//! Portable KeyFlip n-gram scorer and layout decision engine.
pub mod detect;
pub mod engine;
pub mod ffi;
pub mod keymap;
pub mod model;
pub mod selection;
pub mod tracker;

pub use detect::{Config, Decision, Detector};
pub use engine::Engine;
pub use keymap::{Keymap, Lang};
pub use model::Model;
pub use tracker::{Action, Decider, Key, Mapper, RunTracker, TrackerEvent};
