use types::Scalar;

use self::TransitionState::*;
use self::TransitionDirection::*;

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
            direction: FromTo,
            state_time: 0.0,
        }
    }

    pub fn from(&self) -> Scalar {
        match self.direction {
            FromTo => self.v1,
            ToFrom => self.v2,
        }
    }

    pub fn to(&self) -> Scalar {
        match self.direction {
            FromTo => self.v2,
            ToFrom => self.v1,
        }
    }

    pub fn state(&self) -> TransitionState {
        let to = self.to();
        let from = self.from();

        if to > from {
            if self.current >= to {
                Finished
            } else if self.current <= from {
                Start
            } else {
                InProgress
            }
        } else if self.current <= to {
            Finished
        } else if self.current >= from {
            Start
        } else {
            InProgress
        }
    }

    pub fn is_pristine(&self) -> bool {
        self.state() == Start && self.direction == FromTo
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
        } else if self.current < to {
            self.current = to;
        }
        self
    }

    /// Reverse direction and reset `state_time` if we are Finished
    pub fn reverse(&mut self) -> &mut Self {
        if self.state() == Finished {
            self.state_time = 0.0;
        }
        self.direction = match self.direction {
            FromTo => ToFrom,
            ToFrom => FromTo,
        };
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::TransitionState::*;

    #[test]
    fn transition() {
        let mut t = Transition::new(0.0, 1.0, 1.0);
        assert_eq!(t.state(), Start);
        assert_eq!(t.from(), 0.0);
        assert_eq!(t.to(), 1.0);
        assert_eq!(t.state_time, 0.0);

        assert_eq!(t.advance(0.5).current, 0.5);
        assert_eq!(t.state_time, 0.5);
        assert_eq!(t.state(), InProgress);
        assert_eq!(t.advance(0.5).current, 1.0);
        assert_eq!(t.state(), Finished);
        assert_eq!(t.state_time, 1.0);
        assert_eq!(t.advance(0.5).current, 1.0);
        assert_eq!(t.state_time, 1.5);
        assert_eq!(t.state(), Finished);

        t.reverse();
        assert_eq!(t.state_time, 0.0);

        assert_eq!(t.from(), 1.0);
        assert_eq!(t.to(), 0.0);
        assert_eq!(t.state(), Start);
        assert_eq!(t.current, 1.0);


        assert_eq!(t.advance(0.5).current, 0.5);
        assert_eq!(t.state_time, 0.5);
        assert_eq!(t.state(), InProgress);
        assert_eq!(t.reverse().state_time, 0.5);
        assert_eq!(t.reverse().state_time, 0.5);
        assert_eq!(t.advance(0.5).current, 0.0);
        assert_eq!(t.state(), Finished);

        t.reverse();
        assert_eq!(t.state(), Start);
    }
}
