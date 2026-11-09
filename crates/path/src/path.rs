//! Paths: outlines made of straight lines and curves.

use ow_geometry::{Affine, Point};

/// One step of a path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Element {
    /// Starts a new piece at a point, without drawing anything.
    MoveTo(Point),
    /// A straight line to a point.
    LineTo(Point),
    /// A quadratic Bézier curve: its control point, then its end.
    QuadTo(Point, Point),
    /// A cubic Bézier curve: its two control points, then its end.
    CurveTo(Point, Point, Point),
    /// A straight line back to where the piece started.
    Close,
}

/// A line or a curve of a path together with the point it starts from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment {
    /// A straight line between two points.
    Line(Point, Point),
    /// A quadratic Bézier curve: start, control point, end.
    Quad(Point, Point, Point),
    /// A cubic Bézier curve: start, two control points, end.
    Cubic(Point, Point, Point, Point),
}

impl Segment {
    /// Where the segment starts.
    pub const fn start(self) -> Point {
        match self {
            Self::Line(start, _) | Self::Quad(start, ..) | Self::Cubic(start, ..) => start,
        }
    }

    /// Where the segment ends.
    pub const fn end(self) -> Point {
        match self {
            Self::Line(_, end) | Self::Quad(_, _, end) | Self::Cubic(_, _, _, end) => end,
        }
    }

    /// The point a share of the way along, by the curve's own measure: the
    /// start at 0 and the end at 1.
    pub fn point_at(self, share: f64) -> Point {
        match self {
            Self::Line(start, end) => start.lerp(end, share),
            Self::Quad(start, control, end) => {
                let (first, second) = (start.lerp(control, share), control.lerp(end, share));
                first.lerp(second, share)
            }
            Self::Cubic(start, control1, control2, end) => {
                // De Casteljau: a cubic is a quadratic between the points a
                // share of the way along its three legs.
                let legs = [(start, control1), (control1, control2), (control2, end)];
                let [first, second, third] = legs.map(|(from, to)| from.lerp(to, share));
                Self::Quad(first, second, third).point_at(share)
            }
        }
    }
}

/// An outline made of straight lines and curves, in one or more pieces.
///
/// It describes a shape and draws nothing: filling it, stroking it and
/// asking whether a point is inside are done by others, from the same path.
///
/// A path is built the way an HTML canvas builds one, or read from SVG path
/// data, which is also how it prints:
///
/// ```
/// use ow_path::Path;
///
/// let mut roof = Path::new();
/// roof.move_to((0.0, 40.0)).line_to((30.0, 10.0)).line_to((60.0, 40.0));
/// roof.close();
/// assert_eq!(roof.to_string(), "M0 40 L30 10 L60 40 Z");
/// assert_eq!("M0 40 l30 -30 30 30 z".parse(), Ok(roof));
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Path {
    elements: Vec<Element>,
    /// Where the piece being drawn started, once there is one.
    start: Option<Point>,
    /// Where the pen is: the end of the last step.
    current: Option<Point>,
    /// Whether the last piece was closed and the next line begins a new one.
    closed: bool,
}

impl Path {
    /// A path with nothing in it.
    pub const fn new() -> Self {
        Self {
            elements: Vec::new(),
            start: None,
            current: None,
            closed: false,
        }
    }

    /// The steps of the path. Every piece begins with a `MoveTo`.
    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    /// Whether the path has no steps.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Where the pen is: the point the next line or curve starts from, or
    /// `None` on an empty path.
    pub const fn current(&self) -> Option<Point> {
        self.current
    }

    /// Starts a new piece at a point.
    pub fn move_to(&mut self, to: impl Into<Point>) -> &mut Self {
        let to = to.into();
        self.elements.push(Element::MoveTo(to));
        (self.start, self.current) = (Some(to), Some(to));
        self.closed = false;
        self
    }

    /// Draws a straight line to a point. On an empty path the point starts
    /// the first piece.
    pub fn line_to(&mut self, to: impl Into<Point>) -> &mut Self {
        let to = to.into();
        if self.begin_piece(to) {
            self.elements.push(Element::LineTo(to));
            self.current = Some(to);
        }
        self
    }

    /// Draws a quadratic Bézier curve past a control point to an end.
    pub fn quad_to(&mut self, control: impl Into<Point>, to: impl Into<Point>) -> &mut Self {
        let (control, to) = (control.into(), to.into());
        self.begin_piece(control);
        self.elements.push(Element::QuadTo(control, to));
        self.current = Some(to);
        self
    }

    /// Draws a cubic Bézier curve past two control points to an end.
    pub fn curve_to(
        &mut self,
        control1: impl Into<Point>,
        control2: impl Into<Point>,
        to: impl Into<Point>,
    ) -> &mut Self {
        let (control1, control2, to) = (control1.into(), control2.into(), to.into());
        self.begin_piece(control1);
        self.elements.push(Element::CurveTo(control1, control2, to));
        self.current = Some(to);
        self
    }

    /// Closes the piece being drawn with a straight line back to its start.
    /// A line drawn after that begins a new piece at the same start.
    pub fn close(&mut self) -> &mut Self {
        if self.start.is_some() && !self.closed {
            self.elements.push(Element::Close);
            self.current = self.start;
            self.closed = true;
        }
        self
    }

    /// Makes sure there is a piece to draw on, the way an HTML canvas does:
    /// after a close the next piece starts where the closed one did, and on
    /// an empty path it starts at `fallback`. Returns whether a piece was
    /// already under way.
    fn begin_piece(&mut self, fallback: Point) -> bool {
        match self.start {
            Some(start) if self.closed => {
                self.move_to(start);
                true
            }
            Some(_) => true,
            None => {
                self.move_to(fallback);
                false
            }
        }
    }

    /// Adds every piece of another path.
    pub fn extend(&mut self, other: &Self) -> &mut Self {
        if !other.is_empty() {
            self.elements.extend_from_slice(&other.elements);
            (self.start, self.current) = (other.start, other.current);
            self.closed = other.closed;
        }
        self
    }

    /// The path with every point moved by a transform. Bézier curves stay
    /// Bézier curves under any affine transform, so nothing is approximated.
    #[must_use]
    pub fn transformed(&self, transform: Affine) -> Self {
        let elements = self.elements.iter().map(|&element| match element {
            Element::MoveTo(to) => Element::MoveTo(transform * to),
            Element::LineTo(to) => Element::LineTo(transform * to),
            Element::QuadTo(control, to) => Element::QuadTo(transform * control, transform * to),
            Element::CurveTo(control1, control2, to) => {
                Element::CurveTo(transform * control1, transform * control2, transform * to)
            }
            Element::Close => Element::Close,
        });
        Self {
            elements: elements.collect(),
            start: self.start.map(|start| transform * start),
            current: self.current.map(|current| transform * current),
            closed: self.closed,
        }
    }

    /// The pieces of the path, each with its lines and curves.
    pub fn pieces(&self) -> Vec<Piece> {
        let mut pieces: Vec<Piece> = Vec::new();
        let mut current = Point::ORIGIN;
        for &element in &self.elements {
            let segment = match element {
                Element::MoveTo(to) => {
                    pieces.push(Piece {
                        start: to,
                        segments: Vec::new(),
                        closed: false,
                    });
                    current = to;
                    continue;
                }
                Element::LineTo(to) => Segment::Line(current, to),
                Element::QuadTo(control, to) => Segment::Quad(current, control, to),
                Element::CurveTo(control1, control2, to) => {
                    Segment::Cubic(current, control1, control2, to)
                }
                Element::Close => {
                    if let Some(piece) = pieces.last_mut() {
                        piece.closed = true;
                    }
                    continue;
                }
            };
            current = segment.end();
            if let Some(piece) = pieces.last_mut() {
                piece.segments.push(segment);
            }
        }
        pieces
    }
}

/// One connected piece of a path.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    /// Where the piece starts.
    pub start: Point,
    /// Its lines and curves, each beginning where the one before ended.
    pub segments: Vec<Segment>,
    /// Whether the piece was closed. The closing line is not among the
    /// segments.
    pub closed: bool,
}

impl Piece {
    /// Where the piece ends: the end of its last segment, or its start when
    /// it has none.
    pub fn end(&self) -> Point {
        self.segments.last().map_or(self.start, |last| last.end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    #[test]
    fn a_path_is_built_the_way_a_canvas_builds_it() {
        // A line on an empty path only places the pen: there is nothing to
        // draw a line from. After a close the pen is back at the start, and
        // the next line begins a new piece there.
        let mut path = Path::new();
        path.line_to((1.0, 1.0)).line_to((5.0, 1.5)).close();
        path.line_to((1.0, 9.0)).close().close();
        assert_eq!(path.to_string(), "M1 1 L5 1.5 Z M1 1 L1 9 Z");
        // A curve on an empty path starts at its first control point.
        let (mut quad, mut cubic) = (Path::new(), Path::new());
        quad.quad_to((2.0, 0.0), (2.0, 2.0));
        cubic.curve_to((3.0, 0.0), (3.0, 3.0), (0.0, 3.5));
        assert_eq!(quad.to_string(), "M2 0 Q2 0 2 2");
        assert_eq!(cubic.to_string(), "M3 0 C3 0 3 3 0 3.5");
        assert!(Path::new().close().is_empty() && !cubic.is_empty());
        let pens = (Path::new().current(), path.current(), cubic.current());
        assert_eq!(pens, (None, Some(at(1.0, 1.0)), Some(at(0.0, 3.5))));
    }

    #[test]
    fn pieces_carry_the_point_each_segment_starts_from() {
        let data = "M1 0 L4 0 Q6 0 6 2 Z M9 9 C11 10 12 11 12 12 M3 3";
        let pieces = data.parse::<Path>().unwrap().pieces();
        let line = Segment::Line(at(1.0, 0.0), at(4.0, 0.0));
        let quad = Segment::Quad(at(4.0, 0.0), at(6.0, 0.0), at(6.0, 2.0));
        assert_eq!(pieces[0].segments, [line, quad]);
        assert_eq!((line.start(), line.end()), (at(1.0, 0.0), at(4.0, 0.0)));
        let cubic = pieces[1].segments[0];
        assert_eq!((quad.start(), cubic.start()), (at(4.0, 0.0), at(9.0, 9.0)));
        let ends: Vec<Point> = pieces.iter().map(Piece::end).collect();
        assert_eq!(ends, [at(6.0, 2.0), at(12.0, 12.0), at(3.0, 3.0)]);
        let closed: Vec<bool> = pieces.iter().map(|piece| piece.closed).collect();
        assert_eq!(closed, [true, false, false]);
    }

    #[test]
    fn a_curve_passes_through_its_ends_and_leans_to_its_controls() {
        // Half way, a quadratic curve is half way between its control point
        // and the middle of its ends.
        let quad = Segment::Quad(at(0.0, 0.0), at(2.0, 4.0), at(4.0, 0.0));
        let on_quad = [0.0, 0.5, 1.0].map(|share| quad.point_at(share));
        assert_eq!(on_quad, [at(0.0, 0.0), at(2.0, 2.0), at(4.0, 0.0)]);
        let cubic = Segment::Cubic(at(0.0, 0.0), at(0.0, 8.0), at(8.0, 8.0), at(8.0, 0.0));
        let on_cubic = [0.25, 0.5, 1.0].map(|share| cubic.point_at(share));
        assert_eq!(on_cubic, [at(1.25, 4.5), at(4.0, 6.0), at(8.0, 0.0)]);
        let line = Segment::Line(at(2.0, 2.0), at(4.0, 6.0));
        assert_eq!(line.point_at(0.25), at(2.5, 3.0));
    }

    #[test]
    fn a_transform_moves_every_point_of_a_path() {
        let path: Path = "M1 1 L2 1 Q3 1 3 2 C3 3 2 3 1 3 Z".parse().unwrap();
        // The moved path goes on from its own start, not from the old one.
        let mut moved = path.transformed(Affine::scale(2.0, 10.0));
        moved.line_to((0.0, 0.0));
        let expected = "M2 10 L4 10 Q6 10 6 20 C6 30 4 30 2 30 Z M2 10 L0 0";
        assert_eq!(moved.to_string(), expected);
        // A path added to another brings its pen along. An empty one does
        // not move it.
        let mut both = Path::new();
        both.move_to((9.0, 9.0)).extend(&path).extend(&Path::new());
        both.line_to((7.0, 7.0));
        assert_eq!(both.to_string(), format!("M9 9 {path} M1 1 L7 7"));
    }
}
