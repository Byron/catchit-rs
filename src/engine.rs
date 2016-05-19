use std::cell::RefCell;

use rand::{self, Rng};
use vecmath::{vec2_len, vec2_sub, vec2_scale, vec2_add, vec2_normalized};

use types::{Object, State, Extent, Scalar, Pt, Position, Velocity, Hunter, Obstacle};
use types::ObstacleKind::*;
use types::CollisionShape::*;
use transition::Transition;
use transition::TransitionState::*;
use transition::TransitionDirection::*;


const MIN_FIELD_MARGIN: Scalar = 30.0;
const FIELD_VELOCITY_COEFF: Scalar = 0.4;
const OBSTACLE_SIZE_COEFF: Scalar = 0.3;
const MIN_OBSTACLE_TO_HUNTER_COEFF: Scalar = 0.1;
const TRANSITION_DURATION: Scalar = 0.5;
const HOLD_INVISIBILITY_DURATION: Scalar = 0.5;
const COLLISION_VELOCITY_COEFF: Scalar = 0.5;
const ATTRACTIVE_FORCE_DURATION: Scalar = 5.0;
const ATTRACTIVE_FORCE_COEFF: Scalar = 1.25;
const SPECIAL_OBSTACLE_PROBABILITY: f32 = 0.1;
const HUNTER_FORCE: Scalar = 1.0 * 0.1;
const HUNTER_FORCE_SIZE_COEFF: Scalar = 1.5;
const SCORE_PER_PREY: Scalar = 10.0;
const SCORE_COEFF_INCREMENT_MULTIPLIER: Scalar = 0.1;
const SPECIAL_OBSTACLE_STATE_SCORE_MULTIPLIER: Scalar = 2.0;

/// The engine implements the game logic
///
/// It relies on user input given as 2d coordinates
pub struct Engine {
    state: Option<State>,
    min_distance: Scalar,
    rng: RefCell<rand::XorShiftRng>,
}

impl Engine {
    fn rnd_obj_pos_in_field(field: &Extent,
                            half_size: Pt,
                            rng: &mut rand::XorShiftRng)
                            -> Position {
        Self::clamp_to_field(field,
                             half_size,
                             [rng.gen_range(0.0, field[0]), rng.gen_range(0.0, field[1])])
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

    fn hunter_half_size(field: &Extent) -> Scalar {
        let margin = (field[0].min(field[1]) * 0.05).max(MIN_FIELD_MARGIN);
        (margin - (MIN_FIELD_MARGIN / 6.0)) / 2.0
    }

    fn state_from_field(field: Extent) -> State {
        assert!(field[0].min(field[1]) >= 320.0,
                "Playing field is too small");
        let half_size = Self::hunter_half_size(&field);

        let mut rng = rand::weak_rng();
        let prey_pos = Self::rnd_obj_pos_in_field(&field, half_size, &mut rng);

        State {
            field: field,
            hunter: Hunter {
                object: Object {
                    pos: [-half_size * 2.0, -half_size * 2.0],
                    half_size: half_size,
                    shape: Circle,
                },
                force: 0.0,
                velocity: [0.0, 0.0],
            },
            prey: Object {
                pos: prey_pos,
                half_size: half_size,
                shape: Square,
            },
            obstacles: Vec::new(),
            obstacle_opacity: Transition::new(1.0, 0.0, TRANSITION_DURATION),
            attracting_force: Transition::new(0.0,
                                              HUNTER_FORCE * ATTRACTIVE_FORCE_COEFF,
                                              TRANSITION_DURATION),
            score: 0,
            score_coeff: 1.0,
            last_dt: 1.0,
        }
    }

    fn set_state(&mut self, state: State) {
        self.min_distance = (state.field[0].powi(2) + state.field[1].powi(2)).sqrt() *
                            MIN_OBSTACLE_TO_HUNTER_COEFF;
        self.state = Some(state);
    }


    fn new_obstacle(rng: &mut rand::XorShiftRng, s: &mut State, min_distance: Scalar) {
        s.prey.pos = Self::rnd_obj_pos_in_field(&s.field, s.prey.half_size, rng);

        let mut half_size = s.hunter.object.half_size * OBSTACLE_SIZE_COEFF;
        let kind = match rng.gen_range(0.0f32, 1.0) {
            p if p < SPECIAL_OBSTACLE_PROBABILITY => {
                half_size *= 2.0;
                if rng.gen_range(0.0f32, 1.0) > 0.5 {
                    InvisibiltySwitch
                } else {
                    AttractiveForceSwitch
                }
            }
            _ => Deadly,
        };
        let vel: Velocity = [rng.gen_range(-s.field[0] * FIELD_VELOCITY_COEFF,
                                           s.field[0] * FIELD_VELOCITY_COEFF),
                             rng.gen_range(-s.field[1] * FIELD_VELOCITY_COEFF,
                                           s.field[1] * FIELD_VELOCITY_COEFF)];

        let mut pos = s.hunter.object.pos;
        while vec2_len(vec2_sub(pos, s.hunter.object.pos)) < min_distance {
            pos = Self::rnd_obj_pos_in_field(&s.field, s.prey.half_size, rng)
        }
        s.obstacles.push(Obstacle {
            kind: kind,
            object: Object {
                pos: Self::clamp_to_field(&s.field, half_size, pos),
                half_size: half_size,
                shape: Circle,
            },
            velocity: vel,
        });
    }

    fn advect_obstacles(s: &mut State, dt: f64) {
        // Move and collide the obstacles.
        for mut obstacle in &mut s.obstacles {

            let obj = &mut obstacle.object;


            let repell_velocity = if s.hunter.force > 0.0 || s.attracting_force.current > 0.0 {
                let vel = vec2_sub(obj.pos, s.hunter.object.pos);
                let velocity_scale = vec2_len(vel) / (s.hunter.object.half_size * 2.0 * 4.0);

                if velocity_scale <= 1.0 {
                    vec2_scale(vel,
                               (1.0 - velocity_scale) *
                               (s.hunter.force - s.attracting_force.current))
                } else {
                    [0.0, 0.0]
                }
            } else {
                [0.0, 0.0]
            };
            obstacle.velocity = vec2_add(obstacle.velocity, repell_velocity);
            obj.pos = vec2_add(obj.pos, vec2_scale(obstacle.velocity, dt));

            if obj.left() <= 0.0 || obj.right() >= s.field[0] {
                obstacle.velocity[0] = -obstacle.velocity[0];
            }

            if obj.top() <= 0.0 || obj.bottom() >= s.field[1] {
                obstacle.velocity[1] = -obstacle.velocity[1];
            }

            obj.pos = Self::clamp_to_field(&s.field, obj.half_size, obj.pos);
        }
    }

    fn pos_out_of_field(field: &Extent, pos: &Position) -> bool {
        pos[0] < 0.0 || pos[0] > field[0] || pos[1] < 0.0 || pos[1] > field[1]
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

            s.last_dt = dt;
            if !Self::pos_out_of_field(&s.field, &s.hunter.object.pos) &&
               vec2_len(s.hunter.velocity) > 10.0 {
                s.score_coeff += SCORE_COEFF_INCREMENT_MULTIPLIER * dt;
            }

            if s.hunter.object.intersects(&s.prey) {
                let mut multiplier = s.score_coeff;
                for transition in &[&s.obstacle_opacity, &s.attracting_force] {
                    if !transition.is_pristine() {
                        multiplier *= SPECIAL_OBSTACLE_STATE_SCORE_MULTIPLIER;
                    }
                }
                s.score += (SCORE_PER_PREY * multiplier) as u32;
                Self::new_obstacle(&mut self.rng.borrow_mut(), s, self.min_distance);
            }// check hunter-prey intersection

            Self::advect_obstacles(s, dt);

            // advance transitions
            for &mut (ref mut t, duration) in &mut [(&mut s.obstacle_opacity,
                                                     HOLD_INVISIBILITY_DURATION),
                                                    (&mut s.attracting_force,
                                                     ATTRACTIVE_FORCE_DURATION)]
                .iter_mut() {
                match t.state() {
                    Start => {
                        if t.direction == ToFrom {
                            t.reverse();
                        }
                    }
                    InProgress => {
                        t.advance(dt);
                    }
                    Finished => {
                        t.state_time += dt;

                        if t.state_time >= TRANSITION_DURATION + duration {
                            let dir = t.direction.clone();
                            t.reverse();

                            if dir == FromTo {
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
                        Deadly => {
                            is_game_over = true;
                            break;
                        }
                        InvisibiltySwitch | AttractiveForceSwitch => {
                            let new_vel =
                                vec2_scale(vec2_normalized(vec2_sub(obstacle.object.pos,
                                                                    s.hunter.object.pos)),
                                           vec2_len(obstacle.velocity) * COLLISION_VELOCITY_COEFF);
                            obstacle.velocity = vec2_add(new_vel, s.hunter.velocity);
                            let transition = match obstacle.kind {
                                InvisibiltySwitch => &mut s.obstacle_opacity,
                                AttractiveForceSwitch => &mut s.attracting_force,
                                Deadly => unreachable!(),
                            };

                            if transition.state() == Start && transition.direction == FromTo {
                                transition.advance(dt);
                            }
                        }
                    }
                }
            }

            // hunter velocity only remains once we get a move input
            s.hunter.velocity = [0.0, 0.0];
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
            s.hunter.velocity = vec2_scale(vec2_sub(pos, s.hunter.object.pos), 1.0 / s.last_dt);
            s.hunter.object.pos = pos;

            if Self::pos_out_of_field(&s.field, &pos) {
                s.score_coeff = 1.0;
            }
        }
    }

    /// If enabled, a forcefield is created around the hunter, usually repelling
    /// spheres.
    pub fn set_hunter_force(&mut self, enabled: bool) {
        if let Some(ref mut s) = self.state {
            if enabled {
                s.hunter.force = HUNTER_FORCE;
                s.hunter.object.half_size = Self::hunter_half_size(&s.field) *
                                            HUNTER_FORCE_SIZE_COEFF;
            } else {
                s.hunter.force = 0.0;
                s.hunter.object.half_size = Self::hunter_half_size(&s.field);
            }
        }
    }
}
