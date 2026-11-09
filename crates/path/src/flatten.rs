//! Turning curves into straight lines.

use ow_geometry::{Point, Vector};

use crate::path::{Path, Piece, Segment};

/// The most lines one curve is cut into, however fine the tolerance.
const MOST_LINES: u32 = 1024;

/// A piece of a path as corners joined by straight lines.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Polyline {
    /// The corners, in order. A closed polyline does not repeat its first
    /// corner at the end.
    pub points: Vec<Point>,
    /// Whether the last corner is joined back to the first.
    pub closed: bool,
}

impl Polyline {
    /// The straight lines between the corners, as pairs of ends. A closed
    /// polyline has one more: the line back to its first corner.
    pub fn lines(&self) -> impl Iterator<Item = (Point, Point)> + '_ {
        self.joined(self.closed)
    }

    /// The lines between the corners, with the one back to the first corner
    /// when `closed` asks for it, whatever the polyline says itself.
    pub(crate) fn joined(&self, closed: bool) -> impl Iterator<Item = (Point, Point)> + '_ {
        let closing = self.points.first().filter(|_| closed);
        let ends = self.points.iter().skip(1).chain(closing);
        self.points.iter().copied().zip(ends.copied())
    }

    /// How far a point is from the nearest point of the polyline. Infinite
    /// for a polyline without corners.
    pub fn distance_to(&self, point: Point) -> f64 {
        // The nearest point of a line lies a share of the way along it, kept
        // between its ends. A line without length gives no number here and
        // is passed over below: its place is the end of the line before it,
        // or the first corner.
        let to_line = |(start, end): (Point, Point)| {
            let along = end - start;
            let share = (point - start).dot(along) / along.length_squared();
            start.lerp(end, share.clamp(0.0, 1.0)).distance(point)
        };
        let to_corner = self.points.first().map(|&first| first.distance(point));
        self.lines()
            .map(to_line)
            .chain(to_corner)
            .fold(f64::INFINITY, f64::min)
    }
}

fn second_difference(a: Point, b: Point, c: Point) -> Vector {
    (a - b) + (c - b)
}

impl Segment {
    /// How many straight lines, each covering the same share of the curve,
    /// stay within `tolerance` of it.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the value cast is a whole number from 1 to 1024"
    )]
    fn lines_needed(self, tolerance: f64) -> u32 {
        // Wang's bound. A curve cut into n equal shares strays from its
        // chords by no more than `bend / (n * n)`, where the bend comes from
        // how fast the curve's direction changes: its second differences.
        let bend = match self {
            Self::Line(..) => return 1,
            Self::Quad(a, b, c) => second_difference(a, b, c).length() / 4.0,
            Self::Cubic(a, b, c, d) => {
                let (first, second) = (second_difference(a, b, c), second_difference(b, c, d));
                0.75 * first.length().max(second.length())
            }
        };
        // A tolerance of zero or below, or one that is not a number, asks
        // for more than there is to give. Whatever is not a number gives
        // way to the most, and so all of them get the most.
        let lines = (bend / tolerance).sqrt().ceil();
        f64::from(MOST_LINES).min(lines).max(1.0) as u32
    }

    /// Adds the corners of straight lines that stay within `tolerance` of
    /// the segment. The segment's start is taken to be there already.
    pub fn flatten_into(self, tolerance: f64, points: &mut Vec<Point>) {
        let lines = self.lines_needed(tolerance);
        let shares = (1..lines).map(|step| f64::from(step) / f64::from(lines));
        points.extend(shares.map(|share| self.point_at(share)));
        points.push(self.end());
    }
}

impl Piece {
    /// The piece as straight lines that stay within `tolerance` of it.
    pub fn flatten(&self, tolerance: f64) -> Polyline {
        let mut points = vec![self.start];
        for segment in &self.segments {
            segment.flatten_into(tolerance, &mut points);
        }
        // A closed piece that was drawn back to its start has that corner
        // once, not twice.
        if self.closed && points.len() > 1 && points.last() == points.first() {
            points.pop();
        }
        Polyline {
            points,
            closed: self.closed,
        }
    }
}

impl Path {
    /// The path as straight lines that stay within `tolerance` of it, one
    /// polyline for each of its pieces.
    ///
    /// The tolerance is a distance in the path's own units. For something
    /// drawn on a screen, a tenth of a pixel cannot be told from the curve.
    pub fn flatten(&self, tolerance: f64) -> Vec<Polyline> {
        self.pieces()
            .iter()
            .map(|piece| piece.flatten(tolerance))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A quarter circle of radius 100, an arch, a loop that crosses itself
    /// and a curve that doubles back.
    const CURVES: &str = "M100 0 C100 55.2 55.2 100 0 100 M0 0 Q300 500 600 0 \
                          M0 0 C400 300 -300 300 100 0 M0 0 C500 0 -200 10 300 10";

    fn line_counts(data: &str, tolerance: f64) -> Vec<usize> {
        let flat = data.parse::<Path>().unwrap().flatten(tolerance);
        flat.iter().map(|piece| piece.lines().count()).collect()
    }

    #[test]
    fn the_lines_stay_within_the_tolerance_of_the_curve() {
        let path: Path = CURVES.parse().unwrap();
        for tolerance in [5.0, 0.1, 0.002] {
            for (piece, flat) in path.pieces().iter().zip(path.flatten(tolerance)) {
                let curve = piece.segments[0];
                let worst = (0..=4000)
                    .map(|step| flat.distance_to(curve.point_at(f64::from(step) / 4000.0)))
                    .fold(0.0, f64::max);
                assert!(worst <= tolerance, "{curve:?} at {tolerance}: {worst}");
            }
        }
    }

    #[test]
    fn a_finer_tolerance_costs_lines_and_only_as_many_as_the_bound_says() {
        // The quarter circle bends by 34.5: 6 lines keep it within 1, and 19
        // within a tenth.
        assert_eq!(line_counts(CURVES, 1.0), [6, 16, 30, 31]);
        assert_eq!(line_counts(CURVES, 0.1), [19, 50, 93, 95]);
        // A tolerance nothing can meet gets the most there is, and no more.
        for impossible in [0.0, -1.0, f64::NAN] {
            assert_eq!(line_counts(CURVES, impossible)[0], 1024);
        }
        // What is straight is one line, however it is written.
        assert_eq!(line_counts("M0 0 Q5 0 10 0 L9e9 9e9", 1e-9), [2]);
    }

    #[test]
    fn a_path_flattens_piece_by_piece() {
        // Drawn back to its start and then closed, a piece does not list
        // its start twice. A piece closed where it started keeps its one
        // corner.
        let data = "M0 0 L10 0 L10 10 L0 0 Z M20 0 Q30 0 30 10 M5 5 Z";
        let flat = data.parse::<Path>().unwrap().flatten(1.0);
        let shape = |piece: &Polyline| (piece.closed, piece.points.len());
        let shapes: Vec<_> = flat.iter().map(shape).collect();
        assert_eq!(shapes, [(true, 3), (false, 3), (true, 1)]);
        assert_eq!(line_counts(data, 1.0), [3, 2, 1]);
    }

    #[test]
    fn the_distance_to_a_polyline_is_to_its_nearest_point() {
        let path: Path = "M0 0 H10 V10 M0 0 H10 V10 Z M1 1".parse().unwrap();
        let flat = path.flatten(1.0);
        let (open, closed, dot) = (&flat[0], &flat[1], &flat[2]);
        // Beside a line, past an end, nearest to the corner, and between
        // the two lines.
        let from = [(4.0, 3.0), (-3.0, 4.0), (13.0, -4.0), (4.0, 5.0)].map(Point::from);
        let distances = from.map(|point| open.distance_to(point));
        assert_eq!(distances, [3.0, 5.0, 5.0, 5.0]);
        // Closed, the line back from the last corner counts too.
        assert!((closed.distance_to(from[3]) - 0.5_f64.sqrt()).abs() < 1e-12);
        assert_eq!(dot.distance_to(from[3]), 5.0);
        assert_eq!(Polyline::default().distance_to(from[3]), f64::INFINITY);
    }
}
