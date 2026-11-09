//! How things move: the few springs every part of the interface shares.

use ow_motion::Spring;

/// The motion of an interface: a spring for each kind of move.
///
/// Parts do not pick spring constants of their own. They name the kind of
/// move, and the whole interface moves in one manner that can be changed in
/// one place, for instance for someone who asks their system for less
/// motion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motion {
    /// What answers a press right under the pointer: a button, a switch, a
    /// highlight. There in a fifth of a second, without passing its target.
    pub quick: Spring,
    /// What is large or travels far: a panel, a page, a sheet that scrolls
    /// into place. There in two fifths of a second, without passing its
    /// target.
    pub smooth: Spring,
    /// What arrives and should be noticed: a menu that opens, something
    /// dropped into place. It passes its target once, by less than a
    /// twentieth of the way, and is there in half a second.
    pub lively: Spring,
    /// How long something takes to fade in or out, in seconds.
    pub fade: f64,
}

impl Motion {
    /// The motion of the interface.
    pub const STANDARD: Self = Self {
        quick: Spring::preset(0.18, 1.0),
        smooth: Spring::preset(0.35, 1.0),
        lively: Spring::preset(0.4, 0.7),
        fade: 0.12,
    };

    /// The motion for someone who asked for less of it. Nothing travels
    /// visibly and nothing swings: every spring is there within a twelfth
    /// of a second, and what changes is carried by the fade.
    pub const REDUCED: Self = Self {
        quick: Spring::preset(0.07, 1.0),
        smooth: Spring::preset(0.07, 1.0),
        lively: Spring::preset(0.07, 1.0),
        fade: 0.12,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How far a spring that starts at rest, one unit from its target, is
    /// from it after each millisecond of a second: where it is furthest
    /// past the target (below zero), and when it is within a hundredth of
    /// the target for good.
    fn travel(spring: Spring) -> (f64, f64) {
        let left = |millisecond: u32| {
            let moved = spring.transition(f64::from(millisecond) / 1000.0);
            moved.apply(1.0, 0.0).0
        };
        let furthest = (0..1000).map(left).fold(f64::INFINITY, f64::min);
        let last_away = (0..1000).rev().find(|&at| left(at).abs() > 0.01);
        (furthest, f64::from(last_away.unwrap_or(0) + 1) / 1000.0)
    }

    #[test]
    fn each_spring_moves_the_way_its_name_says() {
        let Motion {
            quick,
            smooth,
            lively,
            ..
        } = Motion::STANDARD;
        let ((quick_past, quick_there), (smooth_past, smooth_there)) =
            (travel(quick), travel(smooth));
        assert!(
            quick_past >= 0.0 && quick_there <= 0.2,
            "{quick_past} {quick_there}"
        );
        assert!(
            smooth_past >= 0.0 && smooth_there <= 0.4,
            "{smooth_past} {smooth_there}"
        );
        assert!(quick_there > 0.15 && smooth_there > 0.3);
        let (past, there) = travel(lively);
        assert!(
            past < -0.02 && past > -0.05 && there <= 0.5,
            "{past} {there}"
        );
        for calm in [
            Motion::REDUCED.quick,
            Motion::REDUCED.smooth,
            Motion::REDUCED.lively,
        ] {
            let (past, there) = travel(calm);
            assert!(past >= 0.0 && there <= 1.0 / 12.0, "{past} {there}");
        }
    }
}
