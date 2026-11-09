//! Measuring a path: its bounds, its area, and whether a point is inside.

use ow_geometry::{Point, Rect};

use crate::flatten::Polyline;
use crate::path::{Path, Piece, Segment};

/// Which points count as inside a path that crosses itself or has pieces
/// within pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FillRule {
    /// Inside is wherever the path winds around a point a number of times
    /// that is not zero. A piece drawn the other way round inside another
    /// cuts a hole. It is what fonts and most drawings are made for.
    #[default]
    NonZero,
    /// Inside is wherever a ray from the point crosses the path an odd
    /// number of times, whichever way the pieces run.
    EvenOdd,
}

impl FillRule {
    /// Whether a point the path winds around this many times is inside.
    pub const fn is_inside(self, winding: i32) -> bool {
        match self {
            Self::NonZero => winding != 0,
            Self::EvenOdd => winding % 2 != 0,
        }
    }
}

fn cross(a: Point, b: Point) -> f64 {
    a.to_vector().cross(b.to_vector())
}

impl Polyline {
    /// The area the polyline encloses when its last corner is joined to its
    /// first. It is positive when the corners run clockwise on a screen and
    /// negative the other way round.
    pub fn area(&self) -> f64 {
        let doubled: f64 = self
            .joined(true)
            .map(|(start, end)| cross(start, end))
            .sum();
        doubled / 2.0
    }

    /// How many times the polyline, joined back to its first corner, winds
    /// around a point: once for every clockwise turn on a screen, minus
    /// once for every turn the other way.
    ///
    /// A point on the left or the top of a shape counts as wound around,
    /// and one on its right or its bottom does not, as with
    /// [`Rect::contains`].
    pub fn winding(&self, point: Point) -> i32 {
        // A ray from the point to the right. Every line that crosses it
        // going down the screen winds once around the point, and every line
        // that crosses it going up unwinds once. Counting a line's upper end
        // and not its lower one keeps a corner on the ray from counting
        // twice.
        let crossing = |(start, end): (Point, Point)| {
            let side = (end - start).cross(point - start);
            if start.y <= point.y && point.y < end.y && side > 0.0 {
                1
            } else if end.y <= point.y && point.y < start.y && side < 0.0 {
                -1
            } else {
                0
            }
        };
        self.joined(true).map(crossing).sum()
    }
}

impl Segment {
    /// What the segment adds to the area of a shape it bounds, exactly, by
    /// Green's theorem: the integral of `x dy - y dx` along it, halved.
    fn area(self) -> f64 {
        match self {
            Self::Line(a, b) => cross(a, b) / 2.0,
            Self::Quad(a, b, c) => (2.0 * cross(a, b) + cross(a, c) + 2.0 * cross(b, c)) / 6.0,
            Self::Cubic(a, b, c, d) => {
                let ends = 6.0 * (cross(a, b) + cross(c, d));
                let across = 3.0 * (cross(a, c) + cross(b, c) + cross(b, d)) + cross(a, d);
                (ends + across) / 20.0
            }
        }
    }

    /// The points at which the segment stops moving to the side or stops
    /// moving up or down. Its outermost points are among them and its ends.
    fn turning_points(self) -> impl Iterator<Item = Point> {
        // The roots of `a t^2 + b t + c`, the derivative of one coordinate,
        // in the form that keeps its digits when `a` is small against `b`.
        // With `a` at zero it gives the root of the line that is left. A
        // root that is not there comes out infinite or not a number, and is
        // dropped with the ones outside the curve.
        let roots = |a: f64, b: f64, c: f64| {
            let q = -(b + b.signum() * (b * b - 4.0 * a * c).sqrt()) / 2.0;
            [q / a, c / q]
        };
        let shares = match self {
            Self::Line(..) => Vec::new(),
            Self::Quad(a, b, c) => {
                let (first, second) = (b - a, c - b);
                let sideways = roots(0.0, second.x - first.x, first.x);
                [sideways, roots(0.0, second.y - first.y, first.y)].concat()
            }
            Self::Cubic(a, b, c, d) => {
                let (first, second, third) = (b - a, c - b, d - c);
                let axis = |p: f64, q: f64, r: f64| roots(p - 2.0 * q + r, 2.0 * (q - p), p);
                let sideways = axis(first.x, second.x, third.x);
                [sideways, axis(first.y, second.y, third.y)].concat()
            }
        };
        let on_curve = shares
            .into_iter()
            .filter(|share| (0.0..=1.0).contains(share));
        on_curve.map(move |share| self.point_at(share))
    }
}

impl Path {
    /// The smallest rectangle along the axes that holds the whole path, or
    /// `None` for a path without points.
    ///
    /// It is tight: a curve counts as far as it reaches, not as far as its
    /// control points lie.
    pub fn bounds(&self) -> Option<Rect> {
        let mut bounds: Option<Rect> = None;
        let mut include = |point: Point| {
            let so_far = bounds.unwrap_or(Rect::from_points(point, point));
            bounds = Some(so_far.including(point));
        };
        for piece in self.pieces() {
            include(piece.start);
            for segment in piece.segments {
                include(segment.end());
                segment.turning_points().for_each(&mut include);
            }
        }
        bounds
    }

    /// The area the path fills, with every piece joined back to its start.
    /// A piece that runs clockwise on a screen counts as positive and one
    /// that runs the other way as negative, so a hole takes its area away.
    pub fn area(&self) -> f64 {
        let piece_area = |piece: &Piece| {
            let closing = Segment::Line(piece.end(), piece.start);
            let drawn: f64 = piece.segments.iter().map(|segment| segment.area()).sum();
            drawn + closing.area()
        };
        self.pieces().iter().map(piece_area).sum()
    }

    /// Whether a point is inside the path when it is filled by a rule.
    ///
    /// Curves are measured as straight lines that stay within `tolerance`
    /// of them, so a point closer to a curved edge than that may be told
    /// either way. Each call cuts the path into lines anew: to ask about
    /// many points, [flatten](Self::flatten) it once and add up what
    /// [`Polyline::winding`] says for each piece.
    pub fn contains(&self, point: impl Into<Point>, rule: FillRule, tolerance: f64) -> bool {
        let point = point.into();
        let pieces = self.flatten(tolerance);
        rule.is_inside(pieces.iter().map(|piece| piece.winding(point)).sum())
    }
}

#[cfg(test)]
mod tests {
    use ow_geometry::Insets;

    use super::*;

    #[test]
    fn the_rules_differ_where_a_path_winds_twice_and_agree_on_a_hole() {
        // Two squares drawn clockwise on a screen that share a corner, and
        // inside the first a smaller square drawn the other way.
        let data = "M0 0 H30 V30 H0 Z M20 20 H50 V50 H20 Z M5 5 V15 H15 V5 Z";
        let path: Path = data.parse().unwrap();
        let rules = [FillRule::NonZero, FillRule::EvenOdd];
        let inside = |x, y| rules.map(|rule| path.contains((x, y), rule, 0.1));
        // Wound around once, twice, in the hole, and outside.
        assert_eq!(
            (inside(25.0, 5.0), inside(25.0, 25.0)),
            ([true; 2], [true, false])
        );
        assert_eq!(
            (inside(10.0, 10.0), inside(40.0, 10.0)),
            ([false; 2], [false; 2])
        );
        // The hole runs against the squares, so its 100 comes off the area.
        let flat = path.flatten(0.1);
        let areas: Vec<f64> = flat.iter().map(Polyline::area).collect();
        assert_eq!((path.area(), areas), (1700.0, vec![900.0, 900.0, -100.0]));
        let hole = Point::new(10.0, 10.0);
        assert_eq!((flat[0].winding(hole), flat[2].winding(hole)), (1, -1));
    }

    #[test]
    fn edges_belong_to_a_shape_as_they_do_to_a_rectangle() {
        // The same question asked of the rectangle itself has to get the
        // same answer, on the edges and the corners above all.
        let rect = Rect::new(2.0, 3.0, 8.0, 7.0);
        let path: Path = "M2 3 H8 V7 H2 Z".parse().unwrap();
        for x in 0..=10 {
            for y in 0..=10 {
                let point = Point::new(f64::from(x), f64::from(y));
                let inside = path.contains(point, FillRule::NonZero, 0.1);
                assert_eq!(inside, rect.contains(point), "{point:?}");
            }
        }
    }

    #[test]
    fn the_area_of_curves_is_exact() {
        // Against the same curves cut into lines so fine that their area
        // is right to many digits.
        let data = "M10 5 Q60 -75 110 5 C190 45 130 135 70 95 C-30 205 -50 25 10 65";
        let blob: Path = data.parse().unwrap();
        let lines: f64 = blob.flatten(1e-7).iter().map(Polyline::area).sum();
        assert!(
            (blob.area() - lines).abs() < 0.05,
            "{} {lines}",
            blob.area()
        );
        assert!(lines > 10_000.0);
    }

    #[test]
    fn bounds_reach_as_far_as_the_curve_and_no_further() {
        let bounds = |data: &str| data.parse::<Path>().unwrap().bounds();
        assert_eq!(
            bounds("M0 0 Q5 10 10 0"),
            Some(Rect::new(0.0, 0.0, 10.0, 5.0))
        );
        assert_eq!(bounds("M4 5"), Some(Rect::new(4.0, 5.0, 4.0, 5.0)));
        assert_eq!(bounds(""), None);
        // Against a thousand points on each curve: none of them outside,
        // and one within a thousandth of each side.
        let data = "M10 0 C90 -60 -70 140 30 40 C30 90 80 90 80 40 Q120 0 80 -30";
        let flat = data.parse::<Path>().unwrap().flatten(0.0);
        let start = Rect::new(10.0, 0.0, 10.0, 0.0);
        let reached = flat[0]
            .points
            .iter()
            .fold(start, |so_far, &point| so_far.including(point));
        let (bounds, tight) = (bounds(data).unwrap(), Insets::uniform(1e-3));
        assert_eq!(bounds.union(reached), bounds, "{reached:?}");
        assert_eq!(reached.union(bounds.inset(tight)), reached, "{bounds:?}");
    }
}
