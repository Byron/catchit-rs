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
pub const HOLD_INVISIBILITY_DURATION: Scalar = 0.5;
pub const ATTRACTIVE_FORCE_DURATION: Scalar = 5.0;
pub const ATTRACTIVE_FORCE_COEFF: Scalar = 1.25;
pub const SPECIAL_OBSTACLE_PROBABILITY: f32 = 0.5;
pub const HUNTER_FORCE: Scalar = 1.0 * 0.1;
pub const HUNTER_FORCE_SIZE_COEFF: Scalar = 1.5;

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
    pub velocity: Velocity
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hunter {
    pub object: Object,
    pub force: Scalar
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
    /// transition between opaque and invisible obstacles
    pub obstacle_opacity: Transition,
    /// transition between no attracting force and maximum one
    pub attracting_force: Transition,
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
    /// Keeps timing information, helpful to determine how long a state is held
    pub state_time: f64,
}



impl Transition {
    pub fn new(from: Scalar, to: Scalar, transition_time_s: f64) -> Transition {
        Transition {
            v1: from,
            v2: to,
            current: from,
            transition_time_s: transition_time_s,
            direction: TransitionDirection::FromTo,
            state_time: 0.0,
        }
    }

    pub fn from(&self) -> Scalar {
        match self.direction {
            TransitionDirection::FromTo => self.v1,
            TransitionDirection::ToFrom => self.v2,
        }
    }

    pub fn to(&self) -> Scalar {
        match self.direction {
            TransitionDirection::FromTo => self.v2,
            TransitionDirection::ToFrom => self.v1,
        }
    }

    pub fn state(&self) -> TransitionState {
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
    /// Advances the `state_time` as well
    pub fn advance(&mut self, dt: f64) -> &mut Self {
        self.state_time += dt;

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

    /// Reverse direction and reset `state_time` if we are Finished
    pub fn reverse(&mut self) -> &mut Self {
        if self.state() == TransitionState::Finished {
            self.state_time = 0.0;
        }
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
            hunter: Hunter {
                object: Object {
                    pos: [-half_size * 2.0, -half_size * 2.0],
                    half_size: half_size,
                    shape: CollisionShape::Circle
                },
                force: 0.0
            },
            prey: Object {
                pos: prey_pos,
                half_size: half_size,
                shape: CollisionShape::Square
            }, 
            obstacles: Vec::new(),
            obstacle_opacity: Transition::new(1.0, 0.0, TRANSITION_DURATION),
            attracting_force: Transition::new(0.0, HUNTER_FORCE * ATTRACTIVE_FORCE_COEFF,
                                              TRANSITION_DURATION),
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

        let mut half_size = s.hunter.object.half_size * OBSTACLE_SIZE_COEFF;
        let kind = match rng.gen_range(0.0f32, 1.0) {
            _p if _p > SPECIAL_OBSTACLE_PROBABILITY => {
                half_size *= 2.0;
                if rng.gen_range(0.0f32, 1.0) > 0.5 {
                    ObstacleKind::InvisibiltySwitch
                } else {
                    ObstacleKind::AttractiveForceSwitch
                }
            },
            _               => ObstacleKind::Deadly,
        };
        let vel: Velocity = [
            rng.gen_range(-s.field[0] * FIELD_VELOCITY_COEFF,
                           s.field[0] * FIELD_VELOCITY_COEFF),
            rng.gen_range(-s.field[1] * FIELD_VELOCITY_COEFF,
                           s.field[1] * FIELD_VELOCITY_COEFF),
        ];

        let mut pos = s.hunter.object.pos;
        while vec2_len(vec2_sub(pos, s.hunter.object.pos)) < min_distance {
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


            let repell_velocity = 
                if s.hunter.force > 0.0 || s.attracting_force.current > 0.0 {
                    let vel = vec2_sub(obj.pos, s.hunter.object.pos);
                    let velocity_scale = vec2_len(vel) / (s.hunter.object.half_size * 2.0 * 4.0);

                    if velocity_scale <= 1.0 {
                        vec2_scale(vel, (1.0 - velocity_scale) 
                                        * (s.hunter.force - s.attracting_force.current))
                    } else {
                        [0.0, 0.0]
                    }
                } else {
                    [0.0, 0.0]
                };
            obstacle.velocity = vec2_add(obstacle.velocity, repell_velocity);
            obj.pos = vec2_add(obj.pos, vec2_scale(obstacle.velocity , dt));

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
            if s.hunter.object.intersects(&s.prey) {
                s.score += 10;
                Self::new_obstacle(&mut self.rng.borrow_mut(), s, self.min_distance);
            }// check hunter-prey intersection

            Self::advect_obstacles(s, dt);

            // advance transitions
            for &mut (ref mut t, duration) in 
                                [(&mut s.obstacle_opacity, HOLD_INVISIBILITY_DURATION),
                                 (&mut s.attracting_force, ATTRACTIVE_FORCE_DURATION),]
                                 .iter_mut() {
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
                        t.state_time += dt;

                        if t.state_time >= TRANSITION_DURATION + duration {
                            let dir = t.direction.clone();
                            t.reverse();

                            if dir == TransitionDirection::FromTo {
                                t.advance(dt);
                            }
                        }
                    }
                }
            }

            // Handle obstacle hits
            for obstacle in &mut s.obstacles {
                if obstacle.object.intersects(&s.hunter.object) {
                    match obstacle.kind {
                        ObstacleKind::Deadly => {
                            is_game_over = true;
                            break;
                        },
                         ObstacleKind::InvisibiltySwitch
                        |ObstacleKind::AttractiveForceSwitch => {
                            let transition = match obstacle.kind {
                                ObstacleKind::InvisibiltySwitch => &mut s.obstacle_opacity,
                                ObstacleKind::AttractiveForceSwitch => &mut s.attracting_force,
                                ObstacleKind::Deadly => unreachable!(),
                            };

                            if transition.state() == TransitionState::Start &&
                               transition.direction == TransitionDirection::FromTo {
                                transition.advance(dt);
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
            s.hunter.object.pos = pos;
        }
    }

    /// If enabled, a forcefield is created around the hunter, usually repelling
    /// spheres.
    pub fn set_hunter_force(&mut self, enabled: bool) {
        if let Some(ref mut s) = self.state {
            if enabled {
                s.hunter.force += HUNTER_FORCE;
                s.hunter.object.half_size *= HUNTER_FORCE_SIZE_COEFF;
            } else {
                s.hunter.force -= HUNTER_FORCE;
                s.hunter.object.half_size *= 1.0 / HUNTER_FORCE_SIZE_COEFF;
            }
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
        assert_eq!(t.state_time, 0.0);

        assert_eq!(t.advance(0.5).current, 0.5);
        assert_eq!(t.state_time, 0.5);
        assert_eq!(t.state(), TransitionState::InProgress);
        assert_eq!(t.advance(0.5).current, 1.0);
        assert_eq!(t.state(), TransitionState::Finished);
        assert_eq!(t.state_time, 1.0);
        assert_eq!(t.advance(0.5).current, 1.0);
        assert_eq!(t.state_time, 1.5);
        assert_eq!(t.state(), TransitionState::Finished);

        t.reverse();
        assert_eq!(t.state_time, 0.0);

        assert_eq!(t.from(), 1.0);
        assert_eq!(t.to(), 0.0);  
        assert_eq!(t.state(), TransitionState::Start);
        assert_eq!(t.current, 1.0);


        assert_eq!(t.advance(0.5).current, 0.5);
        assert_eq!(t.state_time, 0.5);
        assert_eq!(t.state(), TransitionState::InProgress);
        assert_eq!(t.reverse().state_time, 0.5);
        assert_eq!(t.reverse().state_time, 0.5);
        assert_eq!(t.advance(0.5).current, 0.0);
        assert_eq!(t.state(), TransitionState::Finished);

        t.reverse();
        assert_eq!(t.state(), TransitionState::Start);
    }
}
