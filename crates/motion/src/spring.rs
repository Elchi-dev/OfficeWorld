//! Damped springs, solved in closed form.
//!
//! A spring here is the equation
//! `mass * x'' + damping * x' + stiffness * x = 0`, where `x` is the distance
//! from the target. Its solution is evaluated directly at the time asked for
//! and never stepped frame by frame, so an animation follows the same path at
//! 60 Hz, at 144 Hz and across a dropped frame.

use std::error::Error;
use std::f64::consts::TAU;
use std::fmt;

use crate::animatable::Animatable;

/// A damped spring that pulls a value towards its target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    stiffness: f64,
    damping: f64,
    mass: f64,
}

/// A spring parameter that cannot describe an animation that ends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidSpring {
    parameter: &'static str,
    value: f64,
    out_of_range: bool,
}

impl fmt::Display for InvalidSpring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            parameter, value, ..
        } = self;
        if self.out_of_range {
            write!(
                f,
                "A spring's {parameter} is {value:e}. Next to its other constants that is too \
                 large or too small to compute the motion."
            )
        } else {
            write!(
                f,
                "A spring's {parameter} must be a finite number above zero, but it is {value}."
            )
        }
    }
}

impl Error for InvalidSpring {}

fn usable(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn above_zero(parameter: &'static str, value: f64) -> Result<f64, InvalidSpring> {
    let out_of_range = false;
    if usable(value) {
        Ok(value)
    } else {
        Err(InvalidSpring {
            parameter,
            value,
            out_of_range,
        })
    }
}

/// Refuses `value` when a number the solution derives from it has left what
/// an f64 holds.
fn in_range(parameter: &'static str, value: f64, derived: f64) -> Result<(), InvalidSpring> {
    let out_of_range = true;
    if usable(derived) {
        Ok(())
    } else {
        Err(InvalidSpring {
            parameter,
            value,
            out_of_range,
        })
    }
}

impl Spring {
    /// A spring from its physical constants.
    ///
    /// # Errors
    ///
    /// Refuses a constant that is zero, negative, infinite or not a number.
    /// That includes a spring without damping: it would swing forever, and
    /// an animation that never ends keeps the screen drawing.
    ///
    /// Also refuses constants so far apart that the solution overflows.
    /// Such a spring would not fail: it would answer with a wrong number
    /// that looks like a right one.
    pub fn new(stiffness: f64, damping: f64, mass: f64) -> Result<Self, InvalidSpring> {
        let spring = Self {
            stiffness: above_zero("stiffness", stiffness)?,
            damping: above_zero("damping", damping)?,
            mass: above_zero("mass", mass)?,
        };
        in_range("stiffness", stiffness, spring.frequency_squared())?;
        in_range("damping", damping, spring.decay() * spring.decay())?;
        Ok(spring)
    }

    /// A spring described the way it feels.
    ///
    /// `response` is how long one swing would take without damping, in
    /// seconds: smaller is quicker. `damping_ratio` says how the spring
    /// settles. Below 1 it swings past the target and back. At 1 a value
    /// that starts from rest arrives as fast as it can without passing the
    /// target, and above 1 it creeps in more slowly. At any ratio a value
    /// that arrives with enough speed still passes the target once.
    ///
    /// # Errors
    ///
    /// Refuses a value that is zero, negative, infinite or not a number, and
    /// one so large or so small that the solution overflows.
    pub fn from_response(response: f64, damping_ratio: f64) -> Result<Self, InvalidSpring> {
        let frequency = TAU / above_zero("response", response)?;
        let damping = 2.0 * above_zero("damping ratio", damping_ratio)? * frequency;
        // Checked here so the error names what the caller passed, and not
        // the stiffness or the damping it would have turned into.
        in_range("response", response, frequency * frequency)?;
        in_range("damping ratio", damping_ratio, damping * damping)?;
        Self::new(frequency * frequency, damping, 1.0)
    }

    /// A spring that is written into the program, described the way
    /// [`from_response`](Self::from_response) describes one:
    /// `const QUICK: Spring = Spring::preset(0.2, 1.0);`.
    ///
    /// It cannot fail, so it can make a constant. A response or a damping
    /// ratio outside a thousandth to a thousand, which is far beyond
    /// anything that is an animation, is brought to the nearest end of that
    /// range, and one that is not a number to its lower end.
    pub const fn preset(response: f64, damping_ratio: f64) -> Self {
        let frequency = TAU / response.max(1e-3).min(1e3);
        let damping = 2.0 * damping_ratio.max(1e-3).min(1e3) * frequency;
        Self {
            stiffness: frequency * frequency,
            damping,
            mass: 1.0,
        }
    }

    /// The damping as a share of the damping that just keeps the spring from
    /// swinging. Below 1 it swings past its target and back.
    pub fn damping_ratio(&self) -> f64 {
        self.damping / (2.0 * (self.stiffness * self.mass).sqrt())
    }

    fn frequency_squared(&self) -> f64 {
        self.stiffness / self.mass
    }

    fn decay(&self) -> f64 {
        self.damping / (2.0 * self.mass)
    }

    /// How the spring carries a state forward over `elapsed` seconds.
    ///
    /// A span that is negative or not a number counts as none. After an
    /// infinite one the spring is at rest.
    pub fn transition(&self, elapsed: f64) -> Transition {
        // `max` also turns a span that is not a number into zero.
        let time = elapsed.max(0.0);
        let frequency_squared = self.frequency_squared();
        let decay = self.decay();
        let discriminant = decay * decay - frequency_squared;
        let envelope = (-decay * time).exp();

        // Every solution is
        //   x(t) = envelope * (x0 * c(t) + (v0 + decay * x0) * s(t))
        // where c is cos and s is sin divided by its frequency for a spring
        // that swings, cosh and sinh divided by its rate for one that
        // creeps, and 1 and t between the two. `swing` is envelope * c and
        // `drift` is envelope * s.
        let (swing, drift) = if discriminant > 0.0 {
            // cosh grows past what an f64 holds while envelope * cosh is
            // still an ordinary number, so written that way a stiff, heavily
            // damped spring is infinity times zero after a few seconds. The
            // two exponentials are multiplied out instead, and both of their
            // exponents are negative. The slow rate is `decay - rate`,
            // written without the subtraction that would cancel its digits.
            let rate = discriminant.sqrt();
            let slow = (-frequency_squared / (decay + rate) * time).exp();
            let fast = (-(decay + rate) * time).exp();
            let drift = slow * -(-2.0 * rate * time).exp_m1() / (2.0 * rate);
            (f64::midpoint(slow, fast), drift)
        } else if envelope == 0.0 {
            // Nothing is left of the motion. Saying so here keeps the cos of
            // infinity and zero times infinity out of the answer.
            (0.0, 0.0)
        } else if discriminant < 0.0 {
            let frequency = (-discriminant).sqrt();
            let angle = frequency * time;
            (envelope * angle.cos(), envelope * angle.sin() / frequency)
        } else {
            (envelope, envelope * time)
        };

        Transition {
            position_from_position: swing + decay * drift,
            position_from_velocity: drift,
            velocity_from_position: -frequency_squared * drift,
            velocity_from_velocity: swing - decay * drift,
        }
    }

    /// Whether a state has so little energy left that it will never again be
    /// further than `tolerance` from the target.
    ///
    /// A damped spring only ever loses energy, so once this is true it stays
    /// true and the animation may stop drawing.
    pub fn is_at_rest<T: Animatable>(&self, displacement: T, velocity: T, tolerance: f64) -> bool {
        // The energy divided by half the stiffness: the square of the
        // distance the state could still reach if all its motion became
        // distance.
        let reach_squared =
            displacement.length_squared() + velocity.length_squared() * self.mass / self.stiffness;
        reach_squared <= tolerance * tolerance
    }
}

/// How a spring carries a state forward over a span of time.
///
/// The spring equation is linear, so the state after the span is a fixed mix
/// of the state before it. The mix depends only on the spring and the span,
/// so one `Transition` moves every part of a point or a colour alike.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transition {
    position_from_position: f64,
    position_from_velocity: f64,
    velocity_from_position: f64,
    velocity_from_velocity: f64,
}

impl Transition {
    /// Carries a distance from the target and a velocity forward, and
    /// returns both as they are after the span.
    pub fn apply<T: Animatable>(&self, displacement: T, velocity: T) -> (T, T) {
        (
            displacement
                .scaled(self.position_from_position)
                .plus(velocity.scaled(self.position_from_velocity)),
            displacement
                .scaled(self.velocity_from_position)
                .plus(velocity.scaled(self.velocity_from_velocity)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_form_solves_the_equation_in_every_regime() {
        // The ratios next to 1 take the formulas for overshooting and for
        // creeping where they divide by a frequency that is almost zero.
        let mut springs = [0.1, 0.7, 1.0 - 1e-9, 1.0, 1.0 + 1e-9, 1.5, 8.0]
            .map(|ratio| Spring::from_response(0.3, ratio).unwrap())
            .to_vec();
        // A mass other than 1, which `from_response` never builds.
        springs.push(Spring::new(350.0, 12.0, 2.5).unwrap());
        for spring in &springs {
            for (x0, v0) in [(1.0, 0.0), (0.0, 5.0), (-2.0, 30.0)] {
                for time in [0.05, 0.4, 1.5] {
                    // Put into the equation it claims to solve, with its
                    // derivatives measured across a short step, the answer
                    // has to leave nothing over.
                    let step = 1e-5;
                    let state = |time| spring.transition(time).apply(x0, v0);
                    let ((x_before, v_before), (x, v), (x_after, v_after)) =
                        (state(time - step), state(time), state(time + step));
                    let speed = (x_after - x_before) / (2.0 * step);
                    let acceleration = (v_after - v_before) / (2.0 * step);
                    let left_over =
                        spring.mass * acceleration + spring.damping * v + spring.stiffness * x;
                    let scale = spring.stiffness * (x0.abs() + v0.abs());
                    assert!(
                        (speed - v).abs() < 1e-6 * scale && left_over.abs() < 1e-6 * scale,
                        "{spring:?} from ({x0}, {v0}) at {time}: speed {speed} for {v}, {left_over} left over"
                    );
                }
            }
        }
    }

    #[test]
    fn heavy_damping_stays_a_number_after_a_long_time() {
        // Here cosh(50000) is infinite and the envelope beside it is zero, so
        // the textbook form of the solution answers NaN. What is left by then
        // is the slow exponential, exp(-stiffness / damping * time) to four
        // places.
        let spring = Spring::new(100.0, 10_000.0, 1.0).unwrap();
        let (x, v) = spring.transition(10.0).apply(1.0, 0.0);
        assert!((x - (-0.1_f64).exp()).abs() < 1e-4 && v < 0.0, "({x}, {v})");

        // Damped so hard that `decay - rate` rounds to zero: computed by
        // subtraction, the slow rate is zero and the spring never arrives.
        let creeping = Spring::new(1.0, 1e9, 1.0).unwrap();
        let (x, _) = creeping.transition(1e9).apply(1.0, 0.0);
        assert!((x - (-1.0_f64).exp()).abs() < 1e-6, "x = {x}");
    }

    #[test]
    fn rest_once_reached_is_kept() {
        // Stopping an animation relies on this: a state at rest never leaves
        // the tolerance again. Crossing the target at speed is not rest, and
        // neither is one coordinate standing still at the far end of a swing.
        let spring = Spring::from_response(0.5, 0.15).unwrap();
        assert!(!spring.is_at_rest(0.0, 1.0, 0.01));
        assert!(!spring.is_at_rest([0.0, 0.02], [0.0, 0.0], 0.01));
        // This speed carries the value 0.004 away at most.
        assert!(spring.is_at_rest(0.0, 0.05, 0.01));
        let mut rested_at = None;
        for step in 0..2_000 {
            let time = f64::from(step) * 0.005;
            let (x, v) = spring.transition(time).apply([3.0_f32, -1.0], [0.0, 40.0]);
            // A point moves as its coordinates would move apart.
            let (alone, _) = spring.transition(time).apply(-1.0_f64, 40.0);
            assert!((f64::from(x[1]) - alone).abs() < 1e-4, "{x:?}, {alone}");
            if spring.is_at_rest(x, v, 0.01) {
                rested_at.get_or_insert(time);
                assert!(x.length_squared() <= 1e-4, "at rest {x:?} away at {time}");
            } else {
                assert_eq!(rested_at, None, "moving again at {time}");
            }
        }
        assert!(rested_at.is_some_and(|time| time > 1.0), "{rested_at:?}");
    }

    #[test]
    fn no_time_and_time_running_backwards_change_nothing() {
        // The formulas grow without bound for a negative span, so a clock
        // that steps back must not reach them.
        let spring = Spring::from_response(0.3, 0.6).unwrap();
        assert_eq!(spring.transition(0.0).apply(7.0, -3.0), (7.0, -3.0));
        assert_eq!(spring.transition(-2.0), spring.transition(0.0));
        assert_eq!(spring.transition(f64::NAN), spring.transition(0.0));
        // After forever every spring has arrived. The formulas say cos of
        // infinity for one that swings and zero times infinity at ratio 1.
        for ratio in [0.6, 1.0, 3.0] {
            let forever = Spring::from_response(0.3, ratio)
                .unwrap()
                .transition(f64::INFINITY);
            assert_eq!(forever.apply(7.0, -3.0), (0.0, 0.0), "ratio {ratio}");
        }
    }

    #[test]
    fn a_spring_that_would_never_end_is_refused() {
        let error = Spring::new(200.0, 0.0, 1.0).unwrap_err().to_string();
        assert_eq!(
            error,
            "A spring's damping must be a finite number above zero, but it is 0."
        );
        // Named as it was given, not as the stiffness it would have become.
        let error = Spring::from_response(0.0, 1.0).unwrap_err().to_string();
        assert!(error.contains("response"), "{error}");
        assert!(Spring::new(f64::NAN, 1.0, 1.0).is_err());
        assert!(Spring::new(1.0, 1.0, f64::INFINITY).is_err());

        // These are finite and above zero, and squaring them overflows: the
        // first answered 0.5 for a spring that had not moved, the second
        // blamed a stiffness of infinity nobody had passed.
        let error = Spring::new(1.0, 1e200, 1.0).unwrap_err().to_string();
        assert!(
            error.starts_with("A spring's damping is 1e200. Next to"),
            "{error}"
        );
        let error = Spring::from_response(1e-160, 1.0).unwrap_err().to_string();
        assert!(error.starts_with("A spring's response is"), "{error}");
        let error = Spring::from_response(0.3, 1e200).unwrap_err().to_string();
        assert!(error.starts_with("A spring's damping ratio is"), "{error}");
        assert!(Spring::new(1e200, 1.0, 1e-200).is_err());
    }

    #[test]
    fn response_and_ratio_describe_the_spring_they_built() {
        let spring = Spring::from_response(0.25, 0.8).unwrap();
        let period = TAU / (spring.stiffness / spring.mass).sqrt();
        assert!((period - 0.25).abs() < 1e-12 && (spring.damping_ratio() - 0.8).abs() < 1e-12);
        // A mass other than 1 counts in the ratio and in what rest means:
        // this speed still carries the heavy value 0.02 away.
        let heavy = Spring::new(100.0, 40.0, 4.0).unwrap();
        assert!((heavy.damping_ratio() - 1.0).abs() < 1e-12);
        assert!(!heavy.is_at_rest(0.0, 0.1, 0.01));
    }

    #[test]
    fn a_preset_is_the_spring_its_description_gives() {
        for (response, ratio) in [(0.2, 1.0), (0.45, 0.7), (1e-3, 1e3), (1e3, 1e-3)] {
            let described = Spring::from_response(response, ratio);
            assert_eq!(Ok(Spring::preset(response, ratio)), described);
        }
        // Past either end, and what is not a number.
        let (quickest, slowest) = (Spring::preset(1e-3, 1e3), Spring::preset(1e3, 1e-3));
        assert_eq!(Spring::preset(0.0, f64::INFINITY), quickest);
        assert_eq!(Spring::preset(1e9, -4.0), slowest);
        let no_number = Spring::preset(f64::NAN, f64::NAN);
        assert_eq!(no_number, Spring::preset(1e-3, 1e-3));
    }
}
