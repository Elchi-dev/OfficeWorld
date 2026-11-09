//! Values that move towards a target, and can be given a new one at any
//! moment.

use crate::animatable::Animatable;
use crate::spring::Spring;

/// A value that a spring moves towards its target.
///
/// The target can change while the value is still on its way. The value then
/// carries on from where it is, at the speed it has, so a panel that is
/// closed while it opens turns around instead of jumping or stalling.
///
/// Times are seconds on any clock, as long as every call uses the same one.
/// A time before the last change, or one that is not a number, counts as
/// the moment of that change.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Animated<T> {
    spring: Spring,
    target: T,
    // The state at `since`: the distance from the target, and the velocity.
    displacement: T,
    velocity: T,
    since: f64,
}

impl<T: Animatable> Animated<T> {
    /// A value standing still.
    pub fn new(value: T, spring: Spring) -> Self {
        Self {
            spring,
            target: value,
            displacement: T::zero(),
            velocity: T::zero(),
            since: 0.0,
        }
    }

    /// Where the value is heading.
    pub fn target(&self) -> T {
        self.target
    }

    /// The value at `now`.
    pub fn value(&self, now: f64) -> T {
        self.target.plus(self.state(now).0)
    }

    /// How fast the value is changing at `now`, per second.
    pub fn velocity(&self, now: f64) -> T {
        self.state(now).1
    }

    /// Whether the value has arrived: at `now` it is within `tolerance` of
    /// its target and cannot leave that range again by itself.
    pub fn is_settled(&self, now: f64, tolerance: f64) -> bool {
        let (displacement, velocity) = self.state(now);
        self.spring.is_at_rest(displacement, velocity, tolerance)
    }

    /// Sends the value to a new target. It starts from where it is at `now`
    /// and keeps its velocity.
    pub fn set_target(&mut self, target: T, now: f64) {
        self.restart(now);
        self.displacement = self.displacement.plus(self.target.minus(target));
        self.target = target;
    }

    /// Gives the value a velocity at `now`, such as the speed of the finger
    /// that let go of it.
    ///
    /// ```
    /// use ow_motion::{Animated, Spring};
    ///
    /// let mut sheet = Animated::new(0.0, Spring::from_response(0.35, 0.8)?);
    /// sheet.set_target(400.0, 0.0);
    /// // A finger catches the sheet on its way. From then on the sheet is
    /// // wherever the finger is.
    /// sheet.jump_to(180.0);
    /// assert_eq!((sheet.value(0.1), sheet.velocity(0.1)), (180.0, 0.0));
    /// // The finger lets go at 900 units a second. The sheet flies on at
    /// // that speed and the spring takes it the rest of the way.
    /// sheet.set_velocity(900.0, 2.0);
    /// sheet.set_target(400.0, 2.0);
    /// assert_eq!((sheet.value(2.0), sheet.velocity(2.0)), (180.0, 900.0));
    /// assert!(sheet.is_settled(5.0, 0.01));
    /// # Ok::<(), ow_motion::InvalidSpring>(())
    /// ```
    pub fn set_velocity(&mut self, velocity: T, now: f64) {
        self.restart(now);
        self.velocity = velocity;
    }

    /// Moves the value with another spring from `now` on, without a jump in
    /// position or velocity.
    ///
    /// ```
    /// use ow_motion::{Animated, Spring};
    ///
    /// let mut scale = Animated::new(1.0_f64, Spring::from_response(0.5, 0.6)?);
    /// scale.set_target(1.2, 0.0);
    /// let (value, velocity) = (scale.value(0.1), scale.velocity(0.1));
    /// // A quicker spring takes over on the way, from the same state.
    /// scale.set_spring(Spring::from_response(0.15, 1.0)?, 0.1);
    /// assert!((scale.value(0.1) - value).abs() < 1e-12);
    /// assert!((scale.velocity(0.1) - velocity).abs() < 1e-12);
    /// // The first spring would still be swinging now.
    /// assert!(scale.is_settled(0.6, 0.001));
    /// # Ok::<(), ow_motion::InvalidSpring>(())
    /// ```
    pub fn set_spring(&mut self, spring: Spring, now: f64) {
        self.restart(now);
        self.spring = spring;
    }

    /// Puts the value somewhere at once and leaves it at rest there.
    pub fn jump_to(&mut self, value: T) {
        *self = Self::new(value, self.spring);
    }

    fn state(&self, now: f64) -> (T, T) {
        self.spring
            .transition(now - self.since)
            .apply(self.displacement, self.velocity)
    }

    /// Makes `now` the moment the stored state describes.
    fn restart(&mut self, now: f64) {
        (self.displacement, self.velocity) = self.state(now);
        // A clock that steps back has moved the state by nothing, so it must
        // not move the start either: the animation would rewind. A time that
        // is not finite is not a moment to start from at all.
        if now.is_finite() {
            self.since = self.since.max(now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_target_keeps_position_and_velocity() {
        // A jump or a stall at this moment is what makes an interrupted
        // animation look broken.
        let mut width = Animated::new(0.0_f64, Spring::from_response(0.4, 0.5).unwrap());
        width.set_target(100.0, 1.0);
        let (value, velocity) = (width.value(1.13), width.velocity(1.13));
        assert!(value > 5.0 && velocity > 50.0, "{value}, {velocity}");
        width.set_target(-60.0, 1.13);
        let jump = (width.value(1.13) - value).abs() + (width.velocity(1.13) - velocity).abs();
        assert!(jump < 1e-9, "jumped by {jump}");
        assert!(!width.is_settled(1.2, 0.01) && width.is_settled(9.0, 0.01));
        assert!((width.value(9.0) + 60.0).abs() < 0.01);
    }

    #[test]
    fn the_path_does_not_depend_on_the_frames_drawn() {
        // A screen that draws 144 frames a second restarts the stored state
        // 144 times where a test that only asks for the end restarts it
        // never. Both have to see the same motion.
        let mut untouched = Animated::new(0.0_f64, Spring::from_response(0.4, 0.5).unwrap());
        untouched.set_target(250.0, 0.0);
        let mut redrawn = untouched;
        for frame in 1..=144 {
            let now = f64::from(frame) / 144.0;
            redrawn.set_target(250.0, now);
            // Neither may a clock that steps back or a time that is no time.
            redrawn.set_target(250.0, now - 0.5);
            redrawn.set_target(250.0, f64::NAN);
            let gap = redrawn.value(now) - untouched.value(now);
            assert!(gap.abs() < 1e-9, "frame {frame} is {gap} off");
        }
    }
}
