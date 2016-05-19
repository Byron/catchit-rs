use vecmath::{self, vec2_sub, vec2_len};

pub type Scalar = f64;

/// [width, height]
pub type Extent = vecmath::Vector2<Scalar>;
/// [x, y]
pub type Position = vecmath::Vector2<Scalar>;
pub type Velocity = vecmath::Vector2<Scalar>;

use transition::Transition;

/// Points on screen. Usually they correspond to pixels, but might not on a
/// `HiDPI` display
pub type Pt = Scalar;

/// Represents a shape used for collision detection
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionShape {
    Square,
    Circle,
}

/// A game object which knows a few things about itself
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub pos: Position,
    pub half_size: Pt,
    pub shape: CollisionShape,
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
        match (&self.shape, &other.shape) {
            (&CollisionShape::Circle, &CollisionShape::Circle) => {
                vec2_len(vec2_sub(self.pos, other.pos)) <= self.half_size + other.half_size
            }
            _ => {
                self.left() <= other.right() && self.right() >= other.left() &&
                self.top() <= other.bottom() && self.bottom() >= other.top()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObstacleKind {
    /// Enables a temporary attractive force
    AttractiveForceSwitch,
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
    pub velocity: Velocity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hunter {
    pub object: Object,
    pub force: Scalar,
    pub velocity: Velocity,
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
    pub hunter: Hunter,
    /// Hunted the player's character
    pub prey: Object,
    /// Obstacles the hunter must avoid to prevent game-over
    pub obstacles: Vec<Obstacle>,
    /// score of the current game
    pub score: u32,
    /// multiply prey score with the given value
    pub score_coeff: Scalar,
    /// transition between opaque and invisible obstacles
    pub obstacle_opacity: Transition,
    /// transition between no attracting force and maximum one
    pub attracting_force: Transition,
    /// Last delta-time during update
    pub last_dt: f64,
}
