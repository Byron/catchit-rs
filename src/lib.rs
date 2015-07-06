//! A library implementing the `catchit` game
//!
extern crate vecmath;
extern crate rand;

use rand::Rng;
use std::cell::RefCell;

pub const MIN_FIELD_MARGIN: Scalar = 30.0;
pub const FIELD_VELOCITY_COEFF: Scalar = 0.2;
pub const OBSTACLE_SIZE_COEFF: Scalar = 0.3;
pub const MIN_OBSTACLE_TO_HUNTER_COEFF: Scalar = 0.1;

pub type Scalar = f64;

/// [width, height]
pub type Extent = vecmath::Vector2<Scalar>;
/// [x, y]
pub type Position = vecmath::Vector2<Scalar>;
pub type Velocity = vecmath::Vector2<Scalar>;

use vecmath::{vec2_len, vec2_sub};

/// Points on screen. Usually they correspond to pixels, but might not on a 
/// HiDPI display
pub type Pt = Scalar;

/// Represents a shape used for collision detection
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionShape {
    Square,
    Circle
}

/// A game object which knows a few things about itself
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub pos: Position,
    pub half_size: Pt,
    pub shape: CollisionShape
}

impl Object {

    pub fn left(&self) -> Scalar {
        self.pos[0] - self.half_size
    }

    pub fn right(&self) -> Scalar {
        self.pos[0] + self.half_size
    }

    pub fn top(&self) -> Scalar {
        self.pos[1] - self.half_size
    }

    pub fn bottom(&self) -> Scalar {
        self.pos[1] + self.half_size
    }

    /// Returns true if both objects intersect
    pub fn intersects(&self, other: &Object) -> bool {
           self.left() <= other.right()
        && self.right() >= other.left()
        && self.top() <= other.bottom() 
        && self.bottom() >= other.top()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObstacleKind {
    /// Causes all other obstacles to hide themselves for a while
    InvisibiltySwitch,
    /// Kills the player
    Deadly,
}

/// An obstacle the hunter can collide with
#[derive(Debug, Clone, PartialEq)]
pub struct Obstacle {
    pub kind: ObstacleKind,
    pub object: Object,
    pub velocity: Velocity
}

/// It maintains the state of the game and expects to be updated with 
/// time-delta information to compute the next state.
///
/// Please note that the coordinates used in the playing field start at 0
/// and grow
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    /// The playing field
    pub field: Extent,
    /// The player's character
    pub hunter: Object,
    /// Hunted the player's character
    pub prey: Object,
    /// Obstacles the hunter must avoid to prevent game-over
    pub obstacles: Vec<Obstacle>,
    /// score of the current game
    pub score: u32,
}

/// The engine implements the game logic
///
/// It relies on user input given as 2d coordinates
pub struct Engine {
    state: Option<State>,
    min_distance: Scalar,
    rng: RefCell<rand::XorShiftRng>,
}

impl Engine {

    fn rnd_obj_pos_in_field(field: &Extent, half_size: Pt, 
                            rng: &mut rand::XorShiftRng) -> Position {
        Self::clamp_to_field(field, half_size, [rng.gen_range(0.0, field[0]),
                                                rng.gen_range(0.0, field[1])])
    }

    fn clamp_to_field(field: &Extent, half_size: Pt, mut pos: Position) -> Position {
        if pos[0] - half_size < 0.0 {
            pos[0] = half_size;
        }
        if pos[0] + half_size > field[0] {
            pos[0] = field[0] - half_size;
        }
        if pos[1] - half_size < 0.0 {
            pos[1] = half_size;   
        }
        if pos[1] + half_size > field[1] {
            pos[1] = field[1] - half_size;   
        }
        pos
    }

    fn state_from_field(field: Extent) -> State {
        assert!(field[0].min(field[1]) >= 320.0, "Playing field is too small");
        let margin = (field[0].min(field[1]) * 0.05).max(MIN_FIELD_MARGIN);
        let half_size = (margin - (MIN_FIELD_MARGIN / 6.0)) / 2.0;

        let mut rng = rand::weak_rng();
        let prey_pos = Self::rnd_obj_pos_in_field(&field, half_size, &mut rng);

        State {
            field: field,
            hunter: Object {
                pos: [-half_size * 2.0, -half_size * 2.0],
                half_size: half_size,
                shape: CollisionShape::Circle
            },
            prey: Object {
                pos: prey_pos,
                half_size: half_size,
                shape: CollisionShape::Square
            }, 
            obstacles: Vec::new(),
            score: 0
        }
    }

    fn set_state(&mut self, state: State) {
        self.min_distance = (state.field[0].powi(2) + state.field[1].powi(2)).sqrt()
                             * MIN_OBSTACLE_TO_HUNTER_COEFF;
        self.state = Some(state);
    }

    pub fn from_field(field: Extent) -> Engine {
        let mut e = Engine {
            state: None,
            min_distance: 0.0,
            rng: RefCell::new(rand::weak_rng()),
        };
        e.set_state(Self::state_from_field(field));
        e
    }

    /// Reset the engine to use the given game-state.
    /// Can be used to setup a new game as well.
    pub fn reset(&mut self, field: Extent) {
        self.set_state(Self::state_from_field(field));
    }

    /// Update the game state.
    ///
    /// If the returned value is the last game-state, it indicates that the player
    /// is game-over.
    pub fn update(&mut self) -> Result<(), State> {
        let mut is_game_over = false;
        if let Some(ref mut s) = self.state {
            if s.hunter.intersects(&s.prey) {
                s.score += 10;
                let mut rng = self.rng.borrow_mut();
                s.prey.pos = Self::rnd_obj_pos_in_field(&s.field, s.prey.half_size, 
                                                         &mut rng);

                let kind = match rng.gen_range(0.0f32, 1.0) {
                    _p if _p > 0.95 => ObstacleKind::InvisibiltySwitch,
                    _ => ObstacleKind::Deadly,
                };
                let vel: Velocity = [
                    rng.gen_range(0.0, s.field[0] * FIELD_VELOCITY_COEFF),
                    rng.gen_range(0.0, s.field[1] * FIELD_VELOCITY_COEFF),
                ];

                let mut pos = s.hunter.pos;
                while vec2_len(vec2_sub(pos, s.hunter.pos)) < self.min_distance {
                    pos = Self::rnd_obj_pos_in_field(&s.field, s.prey.half_size, 
                                                     &mut rng)
                }
                let half_size = s.hunter.half_size * OBSTACLE_SIZE_COEFF;

                s.obstacles.push(Obstacle {
                    kind: kind,
                    object: Object {
                        pos: Self::clamp_to_field(&s.field, half_size, pos),
                        half_size: half_size,
                        shape: CollisionShape::Circle,
                    },
                    velocity: vel,
                });
            }// check hunter-prey intersection

            // If the hunter is hit, the game is over ... 
            for obstacle in &s.obstacles {
                if obstacle.object.intersects   (&s.hunter) {
                    is_game_over = true;
                    break;
                }
            }
        } // end have game state

        if is_game_over {
            Err(self.state.take().unwrap())
        } else {
            Ok(())
        }
    }

    pub fn state(&self) -> &Option<State> {
        &self.state
    }

    /// Position will be clamped into the playing field
    pub fn set_hunter_pos(&mut self, pos: Position) {
        if let Some(ref mut s) = self.state {
            s.hunter.pos = pos;
        }
    }
}
