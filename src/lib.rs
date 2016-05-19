//! A library implementing the `catchit` game
//!
extern crate vecmath;
extern crate rand;

mod engine;
mod transition;
mod types;

pub use types::{Object, CollisionShape, ObstacleKind, State, Extent, Scalar, Pt, Position,
                Velocity, Hunter, Obstacle};
pub use engine::Engine;
pub use transition::{Transition, TransitionState, TransitionDirection};
