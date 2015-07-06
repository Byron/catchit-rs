//! A library implementing the `catchit` game
//!
extern crate vecmath;
extern crate rand;

mod engine;
mod transition;

pub use engine::{Engine, Object, CollisionShape, ObstacleKind, State, Extent, Scalar};
pub use transition::{Transition, TransitionState, TransitionDirection};

