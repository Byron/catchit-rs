//! A library implementing the `catchit` game
//!
extern crate vecmath;
extern crate rand;

use std::default::Default;

pub const MIN_FIELD_MARGIN: Scalar = 30.0;

pub type Scalar = f64;

/// [width, height]
pub type Extent = vecmath::Vector2<Scalar>;
/// [x, y]
pub type Position = vecmath::Vector2<Scalar>;

/// Points on screen. Usually they correspond to pixels, but might not on a 
/// HiDPI display
pub type pt = Scalar;

/// Represents a shape used for collision detection
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionShape {
    Rectangle,
    Circle
}

/// The player character
#[derive(Debug, Clone, PartialEq)]
pub struct Hunter {
    pos: Position,
    size: pt,
}

/// Represents the prey the hunter tries to get to
#[derive(Debug, Clone, PartialEq)]
pub struct Prey {
    pos: Position,
    size: pt,
}

/// Represents a an obstacle the hunter must avoid
#[derive(Debug, Clone, PartialEq)]
pub struct Obstacle {
    pos: Position,
    size: pt,
}

/// It maintains the state of the game and expects to be updated with 
/// time-delta information to compute the next state.
///
/// Please note that the coordinates used in the playing field start at 0
/// and grow
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    /// Amount of points used as a safety area for the hunter
    pub safety_margin: pt,
    /// The playing field
    pub field: Extent,
    /// The player's character
    pub hunter: Hunter,
    /// Hunted the player's character
    pub prey: Prey,
    /// Obstacles the hunter must avoid to prevent game-over
    pub obstacles: Vec<Obstacle>,
    /// score of the current game
    pub score: u32,
}

/// The engine implements the game logic
///
/// It relies on user input given as 2d coordinates
pub struct Engine {
    state: Option<State>
}

impl Engine {

    fn rnd_obj_pos_in_field(field: &Extent, size: pt) -> Position {
        Self::clamp_to_field(field, size, &[0.0, 0.0])
    }

    fn rnd_obj_pos_in_safe_zone(field: &Extent, margin: pt, size: pt) -> Position {
        Self::clamp_to_field(&field, size, &[0.0, 0.0])
    }

    fn clamp_to_field(field: &Extent, size: pt, pos: &Position) -> Position {
        pos.clone()
    }

    fn state_from_field(field: Extent) -> State {
        assert!(field[0].min(field[1]) >= 320.0, "Playing field is too small");
        let margin = (field[0].min(field[1]) * 0.05).max(MIN_FIELD_MARGIN);
        let size = margin - (MIN_FIELD_MARGIN / 6.0);

        let hunter_pos = Self::rnd_obj_pos_in_safe_zone(&field, margin, size);
        let prey_pos = Self::rnd_obj_pos_in_field(&field, size);

        State {
            safety_margin: margin,
            field: field,
            hunter: Hunter {
                pos: hunter_pos,
                size: size,
            },
            prey: Prey {
                pos: prey_pos,
                size: size,
            }, 
            obstacles: Vec::new(),
            score: 0
        }
    }

    pub fn from_field(field: Extent) -> Engine {
        Engine {
            state: Some(Self::state_from_field(field))
        }
    }

    /// Reset the engine to use the given game-state.
    /// Can be used to setup a new game as well.
    pub fn reset(&mut self, field: Extent) {
        self.state = Some(Self::state_from_field(field));
    }

    /// Update the game state.
    ///
    /// If the returned value is the last game-state, it indicates that the player
    /// is game-over.
    pub fn update(&mut self) -> Result<(), State> {
        Ok(())
    }

    pub fn state(&self) -> &Option<State> {
        &self.state
    }

    /// Position will be clamped into the playing field
    pub fn set_hunter_pos(&mut self, pos: Position) {
        if let Some(ref mut s) = self.state {
            s.hunter.pos = Self::clamp_to_field(&s.field, s.hunter.size, &pos);
        }
    }
}