//! Motion that takes a set time: easing curves and tweens.
//!
//! Springs are the default for anything a person moves or interrupts. A
//! tween is for motion whose length is given, such as a fade, or an
//! animation that a presentation file describes by its duration.

use std::error::Error;
use std::fmt;

use crate::animatable::Animatable;

/// A cubic Bézier easing curve from (0, 0) to (1, 1), the form CSS and the
/// office file formats use.
///
/// The horizontal axis is the share of the time that has passed, the
/// vertical axis the share of the way that has been covered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

/// Control points that do not describe an easing curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidCurve {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl fmt::Display for InvalidCurve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "An easing curve needs finite control points with x1 and x2 from 0 to 1, \
             but it got ({}, {}) and ({}, {}).",
            self.x1, self.y1, self.x2, self.y2
        )
    }
}

impl Error for InvalidCurve {}

impl CubicBezier {
    /// The same speed all the way.
    pub const LINEAR: Self = Self::fixed(0.0, 0.0, 1.0, 1.0);
    /// The CSS `ease` curve: a quick start and a long, soft end.
    pub const EASE: Self = Self::fixed(0.25, 0.1, 0.25, 1.0);
    /// The CSS `ease-in` curve: a slow start.
    pub const EASE_IN: Self = Self::fixed(0.42, 0.0, 1.0, 1.0);
    /// The CSS `ease-out` curve: a slow end.
    pub const EASE_OUT: Self = Self::fixed(0.0, 0.0, 0.58, 1.0);
    /// The CSS `ease-in-out` curve: a slow start and a slow end.
    pub const EASE_IN_OUT: Self = Self::fixed(0.42, 0.0, 0.58, 1.0);

    const fn fixed(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// A curve through the control points (`x1`, `y1`) and (`x2`, `y2`).
    ///
    /// # Errors
    ///
    /// Refuses a point that is not finite, and an `x1` or `x2` outside 0 to
    /// 1: such a curve runs backwards in time and has no single value for
    /// some moments. `y1` and `y2` may leave that range, which makes the
    /// curve overshoot.
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Result<Self, InvalidCurve> {
        let in_time = |x: f64| (0.0..=1.0).contains(&x);
        if in_time(x1) && in_time(x2) && y1.is_finite() && y2.is_finite() {
            Ok(Self { x1, y1, x2, y2 })
        } else {
            Err(InvalidCurve { x1, y1, x2, y2 })
        }
    }

    /// The share of the way covered when `progress` of the time has passed.
    ///
    /// Before the start the answer is 0 and after the end it is 1.
    pub fn ease(&self, progress: f64) -> f64 {
        if progress <= 0.0 {
            return 0.0;
        }
        if progress >= 1.0 {
            return 1.0;
        }
        // The curve is given as x(s) and y(s). x(s) never falls, so the s
        // that belongs to a time lies between 0 and 1 and every guess
        // narrows that bracket. Newton's method is taken while it stays
        // inside; where the curve stands still in time its slope is zero,
        // the step leaves the bracket or is not a number, and the bracket is
        // halved instead.
        let (mut low, mut high) = (0.0, 1.0);
        let mut s = progress;
        for _ in 0..64 {
            let error = bezier(s, self.x1, self.x2) - progress;
            if error.abs() < 1e-14 {
                break;
            }
            if error > 0.0 {
                high = s;
            } else {
                low = s;
            }
            let newton = s - error / slope(s, self.x1, self.x2);
            s = if newton > low && newton < high {
                newton
            } else {
                f64::midpoint(low, high)
            };
        }
        bezier(s, self.y1, self.y2)
    }
}

/// One coordinate of a cubic Bézier curve that runs from 0 to 1 past the
/// control values `a` and `b`.
fn bezier(s: f64, a: f64, b: f64) -> f64 {
    let rest = 1.0 - s;
    3.0 * rest * rest * s * a + 3.0 * rest * s * s * b + s * s * s
}

/// How fast that coordinate changes with `s`.
fn slope(s: f64, a: f64, b: f64) -> f64 {
    let rest = 1.0 - s;
    3.0 * rest * rest * a + 6.0 * rest * s * (b - a) + 3.0 * s * s * (1.0 - b)
}

/// A move from one value to another that takes a set time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tween<T> {
    from: T,
    to: T,
    start: f64,
    duration: f64,
    curve: CubicBezier,
}

impl<T: Animatable> Tween<T> {
    /// A move that begins at `start` and takes `duration` seconds. With a
    /// duration that is not above zero it arrives the moment it begins.
    pub fn new(from: T, to: T, start: f64, duration: f64, curve: CubicBezier) -> Self {
        Self {
            from,
            to,
            start,
            // `max` also turns a duration that is not a number into zero.
            duration: duration.max(0.0),
            curve,
        }
    }

    /// Whether the move has arrived at `now`.
    pub fn is_finished(&self, now: f64) -> bool {
        now >= self.start + self.duration
    }

    /// The value at `now`: `from` until the start and exactly `to` from the
    /// end on.
    pub fn value(&self, now: f64) -> T {
        if self.is_finished(now) {
            return self.to;
        }
        let way = self.curve.ease((now - self.start) / self.duration);
        self.from.plus(self.to.minus(self.from).scaled(way))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A Bézier curve built the way it is drawn, by blending neighbouring
    /// points three times, so `bezier` is checked against the construction
    /// and not against its own polynomial.
    fn drawn(s: f64, a: f64, b: f64) -> f64 {
        let blend = |from: f64, to: f64| from + (to - from) * s;
        let (first, second, third) = (blend(0.0, a), blend(a, b), blend(b, 1.0));
        blend(blend(first, second), blend(second, third))
    }

    #[test]
    fn easing_answers_with_the_point_of_the_curve_at_that_time() {
        // Each curve with the digits it is held to and with its way after a
        // quarter of its time, worked out in exact fractions apart from this
        // crate. For the named curves that pins the control points CSS gives
        // them. The last curve stands still in time at its middle: Newton's
        // method divides by zero there, and its way changes infinitely fast,
        // so it is held to fewer digits.
        let curves = [
            (CubicBezier::EASE, 1e-9, 0.408_511),
            (CubicBezier::EASE_IN, 1e-9, 0.093_465),
            (CubicBezier::EASE_OUT, 1e-9, 0.378_138),
            (CubicBezier::EASE_IN_OUT, 1e-9, 0.129_162),
            (CubicBezier::fixed(0.3, -0.6, 0.7, 1.6), 1e-9, 0.000_338),
            (CubicBezier::fixed(1.0, 0.0, 0.0, 1.0), 1e-4, 0.029_725),
        ];
        for (curve, tolerance, quarter) in curves {
            assert!((curve.ease(0.25) - quarter).abs() < 1e-6, "{curve:?}");
            assert_eq!((curve.ease(-0.5), curve.ease(1.5)), (0.0, 1.0));
            for step in 1..1000 {
                let s = f64::from(step) / 1000.0;
                let (time, way) = (drawn(s, curve.x1, curve.x2), drawn(s, curve.y1, curve.y2));
                let eased = curve.ease(time);
                assert!((eased - way).abs() < tolerance, "{curve:?} at {time}");
            }
        }

        // Time creeps at the start of this curve, x = s * s * s. Newton's
        // first step from there lands far outside the curve, and without the
        // bracket it does not find its way back in the steps it has.
        let slow_start = CubicBezier::fixed(0.0, 0.0, 0.0, 1.0);
        assert!((slow_start.ease(1e-12) - 3e-8).abs() < 1e-9);
    }

    #[test]
    fn a_curve_that_runs_backwards_in_time_is_refused() {
        let error = CubicBezier::new(1.2, 0.0, 0.5, 1.0).unwrap_err();
        assert!(error.to_string().ends_with("(1.2, 0) and (0.5, 1)."));
        assert!(CubicBezier::new(0.2, 0.0, -0.1, 1.0).is_err());
        assert!(CubicBezier::new(0.2, f64::NAN, 0.5, 1.0).is_err());
        assert!(CubicBezier::new(0.2, 0.0, 0.5, f64::INFINITY).is_err());
        assert!(CubicBezier::new(0.3, -0.6, 0.7, 1.6).is_ok());
    }

    #[test]
    fn a_tween_holds_its_ends_exactly() {
        // 0.1 + (0.3 - 0.1) is 0.30000000000000004: blended at the end, a
        // tween would stop a hair beside the value it was sent to.
        let tween = Tween::new(0.1_f64, 0.3, 2.0, 0.5, CubicBezier::LINEAR);
        assert_eq!((tween.value(1.0), tween.value(2.5)), (0.1, 0.3));
        assert!((tween.value(2.25) - 0.2).abs() < 1e-12);
        assert!(!tween.is_finished(2.49) && tween.is_finished(2.5));

        // A duration below zero arrives at once. It is not over before it
        // began.
        let instant = Tween::new(0.1, 0.3, 2.0, -1.0, CubicBezier::EASE);
        assert_eq!((instant.value(1.5), instant.value(2.0)), (0.1, 0.3));
    }
}
