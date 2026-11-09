//! Strokes: the shape a pen of some width leaves along a path.

use std::f64::consts::PI;

use ow_geometry::{Affine, Point, Vector};

use crate::path::{Path, Piece};

/// How two lines of a stroke meet at a corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Join {
    /// The outer edges run on until they meet in a point. Where that point
    /// would be too far out, the corner is cut off as for `Bevel`.
    #[default]
    Miter,
    /// The corner is rounded with the radius of the pen.
    Round,
    /// The corner is cut off straight.
    Bevel,
}

/// How a stroke ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cap {
    /// The stroke stops flat, exactly at the end of the path.
    #[default]
    Butt,
    /// The stroke ends in a half circle around the end of the path.
    Round,
    /// The stroke runs on flat by half its width.
    Square,
}

/// The pen a path is drawn with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    /// How wide the pen is.
    pub width: f64,
    /// How corners are joined.
    pub join: Join,
    /// How the ends of an open piece are finished.
    pub cap: Cap,
    /// How far a mitered corner may reach from the path, in widths of the
    /// pen, before it is cut off. At 4, the default of SVG, corners sharper
    /// than about 29 degrees are cut.
    pub miter_limit: f64,
}

impl Stroke {
    /// A pen of a width with mitered corners and flat ends.
    pub const fn new(width: f64) -> Self {
        Self {
            width,
            join: Join::Miter,
            cap: Cap::Butt,
            miter_limit: 4.0,
        }
    }

    /// The same pen with another way of joining corners.
    #[must_use]
    pub const fn with_join(self, join: Join) -> Self {
        Self { join, ..self }
    }

    /// The same pen with another way of finishing ends.
    #[must_use]
    pub const fn with_cap(self, cap: Cap) -> Self {
        Self { cap, ..self }
    }

    /// The same pen with another limit to how far its mitered corners may
    /// reach.
    #[must_use]
    pub const fn with_miter_limit(self, miter_limit: f64) -> Self {
        Self {
            miter_limit,
            ..self
        }
    }
}

/// A corner of the line down the middle of a stroke.
#[derive(Clone, Copy)]
struct Corner {
    point: Point,
    /// Whether the corner is a bend inside a curve, which is always joined
    /// round, and not a corner of the path, which is joined as asked.
    smooth: bool,
}

/// The line down the middle of a stroke: a piece as straight lines, without
/// lines that have no length, since those have no direction to stroke in.
fn centreline(piece: &Piece, tolerance: f64) -> Vec<Corner> {
    let mut last = piece.start;
    let mut corners = vec![Corner {
        point: last,
        smooth: false,
    }];
    let mut points = Vec::new();
    for segment in &piece.segments {
        points.clear();
        segment.flatten_into(tolerance, &mut points);
        for &point in &points {
            if point != last {
                corners.push(Corner {
                    point,
                    smooth: true,
                });
                last = point;
            }
        }
        // Where a segment ends, the path has a corner of its own.
        if let Some(end) = corners.last_mut() {
            end.smooth = false;
        }
    }
    // A closed piece that was drawn back to its start has that corner once.
    if piece.closed && corners.len() > 1 && last == piece.start {
        corners.pop();
    }
    corners
}

/// A pen at work: it adds the shapes that together cover a stroke.
///
/// Every shape it adds is convex and runs the same way round. Filled by the
/// non-zero rule, shapes that overlap simply add up, so however tight a
/// curve is against the width of the pen, no part of the stroke can cancel
/// another out and leave a hole.
struct Pen<'a> {
    half: f64,
    style: Stroke,
    /// The largest angle of an arc that one straight line may stand for.
    arc_step: f64,
    outline: &'a mut Path,
}

impl Pen<'_> {
    fn shape(&mut self, corners: &[Point]) {
        self.outline.move_to(corners[0]);
        for &corner in &corners[1..] {
            self.outline.line_to(corner);
        }
        self.outline.close();
    }

    /// The points of an arc around a centre that starts at `centre + from`
    /// and turns by an angle, clockwise on a screen when it is positive.
    fn arc(&self, centre: Point, from: Vector, angle: f64) -> Vec<Point> {
        let lines = (angle.abs() / self.arc_step).ceil().max(1.0);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a whole number from 1 to 512: a full turn in the smallest steps"
        )]
        let count = lines as u32;
        (0..=count)
            .map(|step| centre + Affine::rotate(angle * f64::from(step) / lines) * from)
            .collect()
    }

    /// The body of the stroke along one straight line.
    fn line(&mut self, from: Point, to: Point, across: Vector) {
        self.shape(&[from + across, to + across, to - across, from - across]);
    }

    /// The outside of a corner: what lies between the ends of the two lines
    /// that meet at it. On the inside the two lines overlap and need
    /// nothing.
    fn join(&mut self, corner: Corner, into: Vector, out: Vector) {
        // Where a path turns left, its outside is on its right. A turn to
        // the right is a turn to the left walked backwards, and is taken
        // that way, so only one kind of corner is left to draw. A path that
        // turns straight back has both sides outside: the half turn around
        // its end covers them.
        let (into, out) = if into.cross(out) > 0.0 {
            (-out, -into)
        } else {
            (into, out)
        };
        let (from, to) = (
            into.perpendicular() * self.half,
            out.perpendicular() * self.half,
        );
        let (here, cosine) = (corner.point, into.dot(out));
        let mut fan = vec![here];
        match if corner.smooth {
            Join::Round
        } else {
            self.style.join
        } {
            Join::Round => fan.extend(self.arc(here, from, -cosine.clamp(-1.0, 1.0).acos())),
            Join::Miter => {
                // The tip lies where the two outer edges meet: out along the
                // middle between them, further the sharper the corner.
                let reach = (2.0 / (1.0 + cosine)).sqrt();
                fan.push(here + from);
                if reach <= self.style.miter_limit {
                    fan.push(here + (from + to) * (reach * reach / 2.0));
                }
                fan.push(here + to);
            }
            Join::Bevel => fan.extend([here + from, here + to]),
        }
        self.shape(&fan);
    }

    /// The end of an open piece, past a point in a direction.
    fn cap(&mut self, end: Point, direction: Vector) {
        let across = direction.perpendicular() * self.half;
        match self.style.cap {
            Cap::Butt => {}
            Cap::Square => self.line(end, end + direction * self.half, across),
            Cap::Round => {
                let half_circle = self.arc(end, across, -PI);
                self.shape(&half_circle);
            }
        }
    }

    /// A piece without any length: a dot. It has no direction, so a flat
    /// end leaves nothing, a square one a square along the axes.
    fn dot(&mut self, at: Point) {
        let (right, down) = (Vector::new(self.half, 0.0), Vector::new(0.0, self.half));
        match self.style.cap {
            Cap::Butt => {}
            Cap::Square => self.line(at - right, at + right, down),
            Cap::Round => {
                // All the way round, the last point is the first again.
                let mut circle = self.arc(at, right, -2.0 * PI);
                circle.pop();
                self.shape(&circle);
            }
        }
    }

    fn piece(&mut self, corners: &[Corner], closed: bool) {
        let count = corners.len();
        if count == 1 {
            self.dot(corners[0].point);
            return;
        }
        let lines = if closed { count } else { count - 1 };
        // The direction of the line that starts at each corner. Corners that
        // repeat the one before are gone, so every line has one.
        let directions: Vec<Vector> = (0..lines)
            .map(|index| corners[(index + 1) % count].point - corners[index].point)
            .map(|along| along.normalized().unwrap_or(Vector::new(1.0, 0.0)))
            .collect();
        for (index, &direction) in directions.iter().enumerate() {
            let next = (index + 1) % count;
            let across = direction.perpendicular() * self.half;
            self.line(corners[index].point, corners[next].point, across);
            // The last line of an open piece has none after it to join.
            if let Some(&onward) = directions.get(next) {
                self.join(corners[next], direction, onward);
            }
        }
        if !closed {
            self.cap(corners[0].point, -directions[0]);
            self.cap(corners[count - 1].point, directions[lines - 1]);
        }
    }
}

impl Path {
    /// The shape a pen leaves along the path: a path that covers the stroke
    /// when it is filled by the non-zero rule.
    ///
    /// Curves, round joins and round ends are made of straight lines that
    /// stay within `tolerance` of them. A pen whose width is not above zero
    /// leaves nothing, and so does a pen that is put down and lifted again
    /// without a line or a close, as in SVG and PDF.
    ///
    /// The result is built to be filled or asked whether it contains a
    /// point. Its pieces overlap, so its area is not the area of the
    /// stroke.
    ///
    /// ```
    /// use ow_path::{Cap, FillRule, Path, Stroke};
    ///
    /// let line = Path::polyline([(10.0, 10.0), (90.0, 10.0)]);
    /// let pen = Stroke::new(8.0).with_cap(Cap::Round);
    /// let stroke = line.stroke(pen, 0.1);
    /// let covers = |x, y| stroke.contains((x, y), FillRule::NonZero, 0.1);
    /// // Four to each side of the line, and round past its ends.
    /// assert!(covers(50.0, 13.9) && !covers(50.0, 14.1));
    /// assert!(covers(93.9, 10.0) && !covers(93.0, 13.0));
    /// ```
    #[must_use]
    pub fn stroke(&self, style: Stroke, tolerance: f64) -> Self {
        let mut outline = Self::new();
        let half = style.width / 2.0;
        if half.is_nan() || half <= 0.0 {
            return outline;
        }
        // The angle at which an arc of the pen's radius strays from its
        // chord by the tolerance. A tolerance nothing can meet gets the
        // smallest step there is.
        let widest = 2.0 * (1.0 - tolerance / half).clamp(-1.0, 1.0).acos();
        let mut pen = Pen {
            half,
            style,
            arc_step: widest.max(PI / 256.0),
            outline: &mut outline,
        };
        for piece in self.pieces() {
            if piece.closed || !piece.segments.is_empty() {
                pen.piece(&centreline(&piece, tolerance), piece.closed);
            }
        }
        outline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::FillRule;

    fn stroked(data: &str, style: Stroke) -> Path {
        data.parse::<Path>().unwrap().stroke(style, 0.01)
    }

    fn covers(stroke: &Path, x: f64, y: f64) -> bool {
        stroke.contains((x, y), FillRule::NonZero, 1.0)
    }

    #[test]
    fn a_round_pen_covers_what_lies_within_half_its_width() {
        // What a stroke with round joins and round ends is: every point no
        // further from the path than half the width of the pen. The path
        // runs straight on through a point, has a sharp corner each way, a
        // line shorter than the pen is wide and a curve that turns tighter
        // than the pen is wide. With it come a closed shape smaller than
        // the pen, a line that turns straight back and a dot. Those are the
        // cases in which the two sides of an outline cross each other.
        let data = "M20 20 H50 H80 L30 45 L34 47 L25 90 C70 60 70 110 60 80 \
                    M134 40 Q130 46 126 40 Q130 34 134 40 Z M100 100 H140 H110 M130 70 Z";
        let path: Path = data.parse().unwrap();
        let round = Stroke::new(16.0).with_join(Join::Round);
        let shapes = path.stroke(round.with_cap(Cap::Round), 0.01).flatten(1.0);
        let middle = path.flatten(0.01);
        for x in 0..=80 {
            for y in 0..=60 {
                let point = Point::new(f64::from(x) * 2.0 + 0.37, f64::from(y) * 2.0 + 0.61);
                let distance = middle.iter().map(|piece| piece.distance_to(point));
                let distance = distance.fold(f64::INFINITY, f64::min);
                let winding: i32 = shapes.iter().map(|shape| shape.winding(point)).sum();
                // Within a twentieth of the edge the lines that stand for
                // the curves decide, either way.
                let near_edge = (distance - 8.0).abs() < 0.05;
                assert!((winding != 0) == (distance < 8.0) || near_edge, "{point:?}");
            }
        }
        // Every shape runs the same way round. One that ran the other way
        // would cancel what it overlaps and leave a hole in the stroke.
        assert!(shapes.iter().all(|shape| shape.area() <= 0.0));
    }

    #[test]
    fn corners_are_joined_as_asked() {
        // A right angle, 10 wide. Its outer corner is at (55, -5).
        let joined = |join| stroked("M0 0 H50 V50", Stroke::new(10.0).with_join(join));
        let (miter, round) = (joined(Join::Miter), joined(Join::Round));
        let bevel = joined(Join::Bevel);
        // Out to the tip, out to the pen's radius, and cut off straight.
        assert!(covers(&miter, 54.9, -4.9) && !covers(&miter, 55.1, -4.9));
        assert!(covers(&round, 53.4, -3.4) && !covers(&round, 53.7, -3.7));
        assert!(covers(&bevel, 52.4, -2.4) && !covers(&bevel, 52.7, -2.7));
        // The inside of the corner is covered whichever way, and nothing is
        // past the flat end.
        for stroke in [&miter, &round, &bevel] {
            assert!(covers(stroke, 46.0, 4.0) && !covers(stroke, 48.0, 51.0));
        }
        // The same corner turned the other way has its tip on the other
        // side.
        let mirrored = stroked("M0 0 H50 V-50", Stroke::new(10.0));
        assert!(covers(&mirrored, 54.9, 4.9) && !covers(&mirrored, 55.1, 4.9));
    }

    #[test]
    fn a_miter_that_would_reach_too_far_is_cut_off() {
        // A corner of about 23 degrees. Its tip would lie 5 widths out.
        let limited = |limit| stroked("M0 0 H100 L0 42", Stroke::new(4.0).with_miter_limit(limit));
        let (cut, whole) = (limited(4.0), limited(6.0));
        assert!(covers(&cut, 100.2, 0.0) && !covers(&cut, 100.6, 0.0));
        assert!(covers(&whole, 106.0, -1.5) && !covers(&whole, 112.0, -1.9));
    }

    #[test]
    fn a_closed_piece_is_joined_all_the_way_round() {
        // A triangle. The corner it starts and ends in gets its tip like
        // the other two, and there is no end to finish.
        let style = Stroke::new(8.0).with_cap(Cap::Square);
        let closed = stroked("M0 0 V60 L60 0 Z", style);
        assert!(covers(&closed, -3.9, -3.9) && covers(&closed, 30.0, -3.9));
        assert!(!covers(&closed, 10.0, 10.0) && !covers(&closed, -4.1, -4.1));
        // Left open, the same lines begin with an end, which a square one
        // pushes out by half the width, and the last side is missing.
        let open = stroked("M0 0 V60 L60 0", style);
        assert!(covers(&open, 3.9, -3.9) && !covers(&open, 30.0, -3.9));
        // A corner drawn twice is one corner, and a triangle drawn back to
        // its start before it is closed is the same triangle.
        assert_eq!(stroked("M0 0 V60 V60 L60 0 L0 0 Z", style), closed);
    }

    #[test]
    fn ends_are_finished_as_asked() {
        // A line 6 wide from (10, 10) to (40, 10): for each kind of end,
        // two points it has to cover and two it must not.
        let ended = |data, cap| stroked(data, Stroke::new(6.0).with_cap(cap));
        let ends = [
            (
                Cap::Butt,
                [(39.9, 12.9), (10.1, 7.1)],
                [(40.1, 10.0), (9.9, 10.0)],
            ),
            (
                Cap::Round,
                [(42.9, 10.0), (7.1, 10.0)],
                [(42.5, 12.5), (7.5, 7.5)],
            ),
            (
                Cap::Square,
                [(42.9, 12.9), (7.1, 7.1)],
                [(43.1, 10.0), (6.9, 10.0)],
            ),
        ];
        for (cap, inside, outside) in ends {
            let line = ended("M10 10 H40", cap);
            assert!(inside.iter().all(|&(x, y)| covers(&line, x, y)), "{cap:?}");
            assert!(
                !outside.iter().any(|&(x, y)| covers(&line, x, y)),
                "{cap:?}"
            );
        }
        // A piece without length is a dot. It has no direction: flat ends
        // leave nothing of it, and square ones a square along the axes.
        let (round, square) = (
            ended("M5 5 L5 5", Cap::Round),
            ended("M5 5 L5 5", Cap::Square),
        );
        assert!(ended("M5 5 L5 5", Cap::Butt).is_empty());
        assert!(covers(&round, 7.9, 5.0) && !covers(&round, 7.5, 7.5));
        assert!(covers(&square, 7.9, 7.9) && !covers(&square, 8.1, 5.0));
        // A pen that is only put down leaves nothing. Closed where it
        // stands, it leaves the same dot.
        assert!(ended("M5 5", Cap::Round).is_empty());
        assert_eq!(ended("M5 5 Z", Cap::Round), round);
    }

    #[test]
    fn a_curve_is_round_inside_and_joined_as_asked_where_it_ends() {
        // A line that runs into a curve, cut so coarsely that the curve is
        // two lines with a bend of 53 degrees between them. Where the line
        // meets the curve the path has a corner, and it gets its tip. The
        // bend is the curve's own and gets none.
        let path: Path = "M0 0 H50 Q50 100 150 100".parse().unwrap();
        let stroke = path.stroke(Stroke::new(20.0), 20.0);
        assert!(covers(&stroke, 56.1, -8.5) && !covers(&stroke, 67.6, 82.4));
    }

    #[test]
    fn arcs_are_cut_into_as_few_lines_as_the_tolerance_allows() {
        // A dot 100 wide. A line that stays within a half of its edge spans
        // 16.2 degrees at most, so it takes 23 to go round.
        let dot: Path = "M0 0 Z".parse().unwrap();
        let round = Stroke::new(100.0).with_cap(Cap::Round);
        let lines = |tolerance| dot.stroke(round, tolerance).flatten(1.0)[0].lines().count();
        assert_eq!(lines(0.5), 23);
        // A tolerance nothing can meet still gives a stroke, as fine as it
        // goes. A pen without width leaves nothing.
        assert_eq!((lines(0.0), lines(f64::NAN)), (512, 512));
        for width in [0.0, -2.0, f64::NAN] {
            assert!(stroked("M0 0 H9", Stroke::new(width)).is_empty(), "{width}");
        }
    }
}
