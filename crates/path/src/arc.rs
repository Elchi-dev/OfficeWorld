//! Arcs: parts of an ellipse, drawn as cubic curves.

use std::f64::consts::{FRAC_PI_2, TAU};

use ow_geometry::{Affine, Point, Vector};

use crate::path::Path;

impl Path {
    /// Draws a part of an ellipse from where the pen is to a point, the way
    /// SVG path data describes an arc: by the two radii of the ellipse, the
    /// angle in radians by which its first axis is turned clockwise on a
    /// screen, and two choices. Two ellipses of that size pass through both
    /// points, and on each there is a short way and a long way round.
    /// `large` takes the long way, and `clockwise` the way that turns
    /// clockwise on a screen.
    ///
    /// Radii too small to reach from one point to the other grow until they
    /// just do. Without a radius the arc is a straight line, and to the
    /// point the pen is at there is no arc at all.
    ///
    /// The arc is made of cubic curves, one for each quarter turn or part
    /// of one. They stray from the ellipse by less than three
    /// ten-thousandths of its radius.
    pub fn arc_to(
        &mut self,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        large: bool,
        clockwise: bool,
        to: impl Into<Point>,
    ) -> &mut Self {
        let to = to.into();
        if self.current() == Some(to) {
            return self;
        }
        // Seen through `unshape`, the ellipse is a circle of radius 1. With
        // no point to start from, no radius or numbers that cannot be
        // worked with, there is no such view, and a straight line is left.
        let shape = Affine::rotate(rotation) * Affine::scale(radius_x.abs(), radius_y.abs());
        let Some((from, unshape)) = self.current().zip(shape.inverse()) else {
            return self.line_to(to);
        };
        // The way from one point to the other as the circle sees it. A
        // chord longer than a diameter does not fit: then the circle grows,
        // which is the chord shrinking, until the chord is a diameter.
        let chord = unshape * (to - from);
        let grow = (chord.length() / 2.0).max(1.0);
        let (shape, chord) = (shape * Affine::scale(grow, grow), chord / grow);
        let Some(along) = chord.normalized() else {
            return self.line_to(to);
        };
        // The centre lies beside the middle of the chord, by what the half
        // chord leaves of the radius: on the right of the way for a short
        // turn clockwise and for a long turn the other way.
        let half = chord.length() / 2.0;
        let aside = (1.0 - half * half).max(0.0).sqrt();
        let side = if large == clockwise { -aside } else { aside };
        let centre = chord / 2.0 + along.perpendicular() * side;
        let angle = |spoke: Vector| spoke.y.atan2(spoke.x);
        let (first, last) = (angle(-centre), angle(chord - centre));
        let turn = if clockwise {
            (last - first).rem_euclid(TAU)
        } else {
            -(first - last).rem_euclid(TAU)
        };
        // One curve for each quarter turn. A quarter turn that comes out a
        // hair too long through rounding is still one curve.
        let curves = (turn.abs() / FRAC_PI_2 - 1e-9).ceil().clamp(1.0, 4.0);
        let step = turn / curves;
        // How far along its tangents the control points of a cubic curve
        // sit for it to follow a circle of radius 1 through an angle.
        let reach = 4.0 / 3.0 * (step / 4.0).tan();
        let place = Affine::translate(from.to_vector()) * shape * Affine::translate(centre);
        let spoke = |angle: f64| Vector::new(angle.cos(), angle.sin());
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a whole number from 1 to 4"
        )]
        let count = curves as u32;
        for index in 1..=count {
            let leave = spoke(first + step * f64::from(index - 1));
            let arrive = spoke(first + step * f64::from(index));
            let control1 = place * (leave + leave.perpendicular() * reach).to_point();
            let control2 = place * (arrive - arrive.perpendicular() * reach).to_point();
            // The last curve ends in the point asked for, to the last digit.
            let end = if index == count {
                to
            } else {
                place * arrive.to_point()
            };
            self.curve_to(control1, control2, end);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;
    use crate::path::Segment;

    #[test]
    fn an_arc_follows_the_ellipse_its_flags_choose() {
        // An ellipse turned by half a radian, and two points on it a radian
        // apart. A second ellipse of the same size passes through both: the
        // first one mirrored at the middle between them. Each of the four
        // arcs keeps to one of the two, and half way along it is at the
        // angle half way.
        let shape = Affine::rotate(0.5) * Affine::scale(30.0, 20.0);
        let on = |centre: Point, angle: f64| centre + shape * Vector::new(angle.cos(), angle.sin());
        let first = Point::new(50.0, 40.0);
        let (from, to) = (on(first, 0.3), on(first, 1.3));
        let second = from + (to - first);
        let arcs = [
            (false, true, first, 0.8, 1_u32),
            (true, false, first, 0.8 + PI, 4),
            (false, false, second, 0.8 + PI, 1),
            (true, true, second, 0.8, 4),
        ];
        let unshape = shape.inverse().unwrap();
        for (large, clockwise, centre, middle, curves) in arcs {
            let mut path = Path::new();
            path.move_to(from);
            path.arc_to(30.0, -20.0, 0.5, large, clockwise, to);
            let segments = &path.pieces()[0].segments;
            let exact = segments.len() == curves as usize && path.current() == Some(to);
            let off = |point: Point| ((unshape * (point - centre)).length() - 1.0).abs();
            let shares = [0.0, 0.25, 0.5, 0.75, 1.0];
            let sampled = |curve: &Segment| shares.map(|share| off(curve.point_at(share)));
            let worst = segments.iter().flat_map(sampled).fold(0.0, f64::max);
            let half_way = f64::from(curves) / 2.0;
            let there = segments[curves as usize / 2].point_at(half_way.fract());
            let astray = there.distance(on(centre, middle));
            assert!(
                exact && worst < 2.8e-4 && astray < 0.01,
                "{large} {clockwise}"
            );
        }
    }

    #[test]
    fn an_arc_that_cannot_be_drawn_as_asked_is_drawn_as_near_as_can_be() {
        // Radii too small to reach grow until the way across is a diameter:
        // a half circle, clockwise over the top, in two quarter turns.
        let mut path = Path::new();
        path.move_to((0.0, 0.0));
        path.arc_to(1.0, 1.0, 0.0, false, true, (10.0, 0.0));
        let segments = &path.pieces()[0].segments;
        let top = segments[0].end().distance(Point::new(5.0, -5.0));
        assert!(segments.len() == 2 && top < 1e-12, "{top}");
        // On an empty path the pen is placed. Without a radius the arc is a
        // straight line, and to the point the pen is at there is none.
        let mut line = Path::new();
        line.arc_to(5.0, 5.0, 0.0, false, true, (3.0, 4.0));
        line.arc_to(0.0, 5.0, 0.0, false, true, (20.0, 0.0));
        line.arc_to(5.0, 5.0, 0.0, true, true, (20.0, 0.0));
        assert_eq!(line.to_string(), "M3 4 L20 0");
    }
}
