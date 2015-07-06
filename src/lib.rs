//! A library implementing the `catchit` game
//!
extern crate vecmath;
extern crate rand;

use rand::Rng;
use std::cell::RefCell;

pub const MIN_FIELD_MARGIN: Scalar = 30.0;
pub const FIELD_VELOCITY_COEFF: Scalar = 0.4;
pub const OBSTACLE_SIZE_COEFF: Scalar = 0.3;
pub const MIN_OBSTACLE_TO_HUNTER_COEFF: Scalar = 0.1;
pub const TRANSITION_DURATION: Scalar =  0.5;

pub type Scalar = f64;

/// [width, height]
pub type Extent = vecmath::Vector2<Scalar>;
/// [x, y]
pub type Position = vecmath::Vector2<Scalar>;
pub type Velocity = vecmath::Vector2<Scalar>;

use vecmath::{vec2_len, vec2_sub, vec2_scale, vec2_add};

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
        match (&self.shape, &other.shape) {
            (&CollisionShape::Circle, &CollisionShape::Circle) => {
                vec2_len(vec2_sub(self.pos, other.pos)) <= self.half_size + other.half_size
            },
            _ => {
                   self.left() <= other.right()
                && self.right() >= other.left()
                && self.top() <= other.bottom() 
                && self.bottom() >= other.top()
            }
        }
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
    /// transition between opaque and invisible obstacles
    pub obstacle_opacity: Transition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransitionState {
    /// Transition is at start, which is the case right after calling `new()`
    Start,
    /// We are neither finished, nor at the start
    InProgress,
    /// We are finished
    Finished,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransitionDirection {
    FromTo,
    ToFrom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub v1: Scalar,
    pub v2: Scalar,
    pub current: Scalar,
    /// Time it takes to make transition
    pub transition_time_s: f64,
    pub direction: TransitionDirection,
}



impl Transition {
    fn new(from: Scalar, to: Scalar, transition_time_s: f64) -> Transition {
        Transition {
            v1: from,
            v2: to,
            current: from,
            transition_time_s: transition_time_s,
            direction: TransitionDirection::FromTo
        }
    }

    fn from(&self) -> Scalar {
        match self.direction {
            TransitionDirection::FromTo => self.v1,
            TransitionDirection::ToFrom => self.v2,
        }
    }

    fn to(&self) -> Scalar {
        match self.direction {
            TransitionDirection::FromTo => self.v2,
            TransitionDirection::ToFrom => self.v1,
        }
    }

    fn state(&self) -> TransitionState {
        let to = self.to();
        let from = self.from();

        if to > from {
            if self.current >= to {
                TransitionState::Finished
            } else if self.current <= from {
                TransitionState::Start
            } else {
                TransitionState::InProgress
            }
        } else {
            if self.current <= to {
                TransitionState::Finished
            } else if self.current >= from {
                TransitionState::Start
            } else {
                TransitionState::InProgress
            }
        }
    }

    /// Move transition towards the finished state
    /// `dt` is in seconds and signals the passed time since the last advance 
    /// call.
    fn advance(&mut self, dt: f64) -> &mut Self {
        let to = self.to();
        let from = self.from();

        let delta = to - from;
        self.current += delta * (dt / self.transition_time_s);

        if to > from {
            if self.current > to {
                self.current = to;
            }
        } else {
            if self.current < to {
                self.current = to;
            }
        }
        self
    }

    fn reverse(&mut self) -> &mut Self {
        self.direction = match self.direction {
            TransitionDirection::FromTo => TransitionDirection::ToFrom,
            TransitionDirection::ToFrom => TransitionDirection::FromTo,
        };
        self
    }
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
            obstacle_opacity: Transition::new(1.0, 0.0, TRANSITION_DURATION),
            score: 0
        }
    }

    fn set_state(&mut self, state: State) {
        self.min_distance = (state.field[0].powi(2) + state.field[1].powi(2)).sqrt()
                             * MIN_OBSTACLE_TO_HUNTER_COEFF;
        self.state = Some(state);
    }


    fn new_obstacle(rng: &mut rand::XorShiftRng, s: &mut State, min_distance: Scalar) {
        s.prey.pos = Self::rnd_obj_pos_in_field(&s.field, s.prey.half_size, rng);

        let mut half_size = s.hunter.half_size * OBSTACLE_SIZE_COEFF;
        let kind = match rng.gen_range(0.0f32, 1.0) {
            _p if _p > 0.94 => {
                half_size *= 2.0;
                ObstacleKind::InvisibiltySwitch
            },
            _               => ObstacleKind::Deadly,
        };
        let vel: Velocity = [
            rng.gen_range(-s.field[0] * FIELD_VELOCITY_COEFF,
                           s.field[0] * FIELD_VELOCITY_COEFF),
            rng.gen_range(-s.field[1] * FIELD_VELOCITY_COEFF,
                           s.field[1] * FIELD_VELOCITY_COEFF),
        ];

        let mut pos = s.hunter.pos;
        while vec2_len(vec2_sub(pos, s.hunter.pos)) < min_distance {
            pos = Self::rnd_obj_pos_in_field(&s.field, s.prey.half_size, rng)
        }
        s.obstacles.push(Obstacle {
            kind: kind,
            object: Object {
                pos: Self::clamp_to_field(&s.field, half_size, pos),
                half_size: half_size,
                shape: CollisionShape::Circle,
            },
            velocity: vel,
        });
    }

    fn advect_obstacles(s: &mut State, dt: f64) {
        // Move and collide the obstacles.
        for obstacle in s.obstacles.iter_mut() {

            let obj = &mut obstacle.object;
            obj.pos = vec2_add(obj.pos, vec2_scale(obstacle.velocity, dt));

            if obj.left() <= 0.0 {
                obstacle.velocity[0] = -obstacle.velocity[0];
            } else if obj.right() >= s.field[0] {
                obstacle.velocity[0] = -obstacle.velocity[0];
            }
            if obj.top() <= 0.0 {
                obstacle.velocity[1] = -obstacle.velocity[1];
            } else if obj.bottom() >= s.field[1] {
                obstacle.velocity[1] = -obstacle.velocity[1];
            }

            obj.pos = Self::clamp_to_field(&s.field, obj.half_size, 
                                            obj.pos);
        }
    }
}

impl Engine {

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
    pub fn update(&mut self, dt: f64) -> Result<(), State> {
        let mut is_game_over = false;
        if let Some(ref mut s) = self.state {
            if s.hunter.intersects(&s.prey) {
                s.score += 10;
                Self::new_obstacle(&mut self.rng.borrow_mut(), s, self.min_distance);
            }// check hunter-prey intersection

            Self::advect_obstacles(s, dt);

            // advance transitions
            for t in [&mut s.obstacle_opacity].iter_mut() {
                match t.state() {
                    TransitionState::Start => {
                        if t.direction == TransitionDirection::ToFrom {
                            t.reverse();
                        }
                    }
                    TransitionState::InProgress => {
                        t.advance(dt);
                    }
                    TransitionState::Finished => {
                        let dir = t.direction.clone();
                        t.reverse();

                        if dir == TransitionDirection::FromTo {
                            t.advance(dt);
                        }
                    }
                }
            }

            // If the hunter is hit, the game is over ... 
            for obstacle in &s.obstacles {
                if obstacle.object.intersects(&s.hunter) {
                    match obstacle.kind {
                        ObstacleKind::Deadly => {
                            is_game_over = true;
                            break;
                        },
                        ObstacleKind::InvisibiltySwitch => {
                            if s.obstacle_opacity.state() == TransitionState::Start &&
                               s.obstacle_opacity.direction == TransitionDirection::FromTo {
                                s.obstacle_opacity.advance(dt);
                            }
                        },
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition() {
        let mut t = Transition::new(0.0, 1.0, 1.0);
        assert_eq!(t.state(), TransitionState::Start);
        assert_eq!(t.from(), 0.0);
        assert_eq!(t.to(), 1.0);

        assert_eq!(t.advance(0.5).current, 0.5);
        assert_eq!(t.state(), TransitionState::InProgress);
        assert_eq!(t.advance(0.5).current, 1.0);
        assert_eq!(t.state(), TransitionState::Finished);

        t.reverse();

        assert_eq!(t.from(), 1.0);
        assert_eq!(t.to(), 0.0);  
        assert_eq!(t.state(), TransitionState::Start);
        assert_eq!(t.current, 1.0);


        assert_eq!(t.advance(0.5).current, 0.5);
        assert_eq!(t.state(), TransitionState::InProgress);
        assert_eq!(t.advance(0.5).current, 0.0);
        assert_eq!(t.state(), TransitionState::Finished);

        t.reverse();
        assert_eq!(t.state(), TransitionState::Start);
    }
}
