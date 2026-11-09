//! Sizes, rectangles and the insets between them.

use std::ops::{Add, Neg, Sub};

use ow_motion::Animatable;

use crate::point::{Point, Vector};

/// A width and a height.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    /// The extent to the right.
    pub width: f64,
    /// The extent downwards.
    pub height: f64,
}

impl Size {
    /// No extent at all.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// A size from its width and height.
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Whether nothing fits into it: a side is zero, negative or not a
    /// number.
    pub fn is_empty(self) -> bool {
        !(self.width > 0.0 && self.height > 0.0)
    }
}

/// A rectangle with sides along the axes, kept as its top left and its
/// bottom right corner.
///
/// The left and the top edge belong to the rectangle and the right and the
/// bottom edge do not, so rectangles that share an edge share no point and
/// a row of them covers every point exactly once.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// The left edge.
    pub x0: f64,
    /// The top edge.
    pub y0: f64,
    /// The right edge.
    pub x1: f64,
    /// The bottom edge.
    pub y1: f64,
}

impl Rect {
    /// The rectangle at the origin without any extent.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// A rectangle from its left, top, right and bottom edge.
    pub const fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self { x0, y0, x1, y1 }
    }

    /// The rectangle of a size with its top left corner at a point.
    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        Self::new(
            origin.x,
            origin.y,
            origin.x + size.width,
            origin.y + size.height,
        )
    }

    /// The rectangle between two opposite corners, whichever two they are.
    pub fn from_points(a: Point, b: Point) -> Self {
        Self::new(a.x.min(b.x), a.y.min(b.y), a.x.max(b.x), a.y.max(b.y))
    }

    /// The extent to the right.
    pub fn width(self) -> f64 {
        self.x1 - self.x0
    }

    /// The extent downwards.
    pub fn height(self) -> f64 {
        self.y1 - self.y0
    }

    /// The width and the height.
    pub fn size(self) -> Size {
        Size::new(self.width(), self.height())
    }

    /// The top left corner.
    pub const fn origin(self) -> Point {
        Point::new(self.x0, self.y0)
    }

    /// The point in the middle.
    pub fn center(self) -> Point {
        Point::new(
            f64::midpoint(self.x0, self.x1),
            f64::midpoint(self.y0, self.y1),
        )
    }

    /// Whether the rectangle covers no point.
    pub fn is_empty(self) -> bool {
        self.size().is_empty()
    }

    /// Whether a point is inside. The left and the top edge are, the right
    /// and the bottom edge are not.
    pub fn contains(self, point: Point) -> bool {
        point.x >= self.x0 && point.x < self.x1 && point.y >= self.y0 && point.y < self.y1
    }

    /// The part both rectangles cover. It is empty when they do not overlap.
    #[must_use]
    pub fn intersect(self, other: Self) -> Self {
        Self::new(
            self.x0.max(other.x0),
            self.y0.max(other.y0),
            self.x1.min(other.x1),
            self.y1.min(other.y1),
        )
    }

    /// The smallest rectangle that covers everything either one covers. A
    /// rectangle that covers nothing adds nothing, wherever it lies.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        Self::new(
            self.x0.min(other.x0),
            self.y0.min(other.y0),
            self.x1.max(other.x1),
            self.y1.max(other.y1),
        )
    }

    /// The smallest rectangle that holds this one and a point. Unlike a
    /// union with a rectangle that covers nothing, a point always counts:
    /// that is what measuring the reach of a row of points needs.
    #[must_use]
    pub fn including(self, point: Point) -> Self {
        Self::new(
            self.x0.min(point.x),
            self.y0.min(point.y),
            self.x1.max(point.x),
            self.y1.max(point.y),
        )
    }

    /// The rectangle with its edges moved inwards by the insets. Negative
    /// insets move them outwards.
    #[must_use]
    pub fn inset(self, insets: Insets) -> Self {
        Self::new(
            self.x0 + insets.left,
            self.y0 + insets.top,
            self.x1 - insets.right,
            self.y1 - insets.bottom,
        )
    }

    /// The smallest rectangle on whole coordinates that covers this one:
    /// the pixels a shape with these bounds can touch.
    #[must_use]
    pub fn round_out(self) -> Self {
        Self::new(
            self.x0.floor(),
            self.y0.floor(),
            self.x1.ceil(),
            self.y1.ceil(),
        )
    }
}

impl Add<Vector> for Rect {
    type Output = Self;

    fn add(self, offset: Vector) -> Self {
        Self::new(
            self.x0 + offset.x,
            self.y0 + offset.y,
            self.x1 + offset.x,
            self.y1 + offset.y,
        )
    }
}

impl Sub<Vector> for Rect {
    type Output = Self;

    fn sub(self, offset: Vector) -> Self {
        self + -offset
    }
}

/// Distances inwards from the four edges of a rectangle: a padding, a
/// margin, the width of a border.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    /// From the top edge.
    pub top: f64,
    /// From the right edge.
    pub right: f64,
    /// From the bottom edge.
    pub bottom: f64,
    /// From the left edge.
    pub left: f64,
}

impl Insets {
    /// No distance on any side.
    pub const ZERO: Self = Self::uniform(0.0);

    /// Insets from the top, right, bottom and left distance, in the order
    /// CSS gives them.
    pub const fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// The same distance on every side.
    pub const fn uniform(distance: f64) -> Self {
        Self::new(distance, distance, distance, distance)
    }

    /// One distance for the left and right side and one for the top and
    /// bottom.
    pub const fn symmetric(horizontal: f64, vertical: f64) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    /// What the insets take from a width and from a height.
    pub fn size(self) -> Size {
        Size::new(self.left + self.right, self.top + self.bottom)
    }
}

impl Neg for Insets {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.top, -self.right, -self.bottom, -self.left)
    }
}

impl Animatable for Size {
    fn zero() -> Self {
        Self::ZERO
    }

    fn plus(self, other: Self) -> Self {
        Self::new(self.width + other.width, self.height + other.height)
    }

    fn scaled(self, factor: f64) -> Self {
        Self::new(self.width * factor, self.height * factor)
    }

    fn length_squared(self) -> f64 {
        self.width * self.width + self.height * self.height
    }
}

// A rectangle is animated edge by edge, which moves and resizes it at once.
impl Animatable for Rect {
    fn zero() -> Self {
        Self::ZERO
    }

    fn plus(self, other: Self) -> Self {
        Self::new(
            self.x0 + other.x0,
            self.y0 + other.y0,
            self.x1 + other.x1,
            self.y1 + other.y1,
        )
    }

    fn scaled(self, factor: f64) -> Self {
        Self::new(
            self.x0 * factor,
            self.y0 * factor,
            self.x1 * factor,
            self.y1 * factor,
        )
    }

    fn length_squared(self) -> f64 {
        [self.x0, self.y0, self.x1, self.y1].length_squared()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOX: Rect = Rect::new(10.0, 20.0, 40.0, 60.0);

    #[test]
    fn a_rectangle_knows_its_measures() {
        let (origin, size) = (Point::new(10.0, 20.0), Size::new(30.0, 40.0));
        assert_eq!(BOX, Rect::from_origin_size(origin, size));
        let (top_right, bottom_left) = (Point::new(40.0, 20.0), Point::new(10.0, 60.0));
        assert_eq!(BOX, Rect::from_points(top_right, bottom_left));
        assert_eq!((BOX.size(), BOX.origin()), (size, origin));
        assert_eq!(BOX.center(), Point::new(25.0, 40.0));
        let offset = Vector::new(5.0, -5.0);
        assert_eq!(BOX + offset, Rect::new(15.0, 15.0, 45.0, 55.0));
        assert_eq!(BOX - offset, Rect::new(5.0, 25.0, 35.0, 65.0));
    }

    #[test]
    fn neighbours_share_an_edge_and_no_point() {
        // Two cells of a sheet side by side: a click on the line between
        // them has to land in exactly one.
        let right_neighbour = BOX + Vector::new(BOX.width(), 0.0);
        let on_the_line = Point::new(40.0, 30.0);
        assert!(!BOX.contains(on_the_line) && right_neighbour.contains(on_the_line));
        assert!(BOX.contains(BOX.origin()) && !BOX.contains(Point::new(10.0, 60.0)));
        assert!(!BOX.contains(Point::new(9.9, 30.0)) && !BOX.contains(Point::new(20.0, 19.9)));
        assert!(BOX.intersect(right_neighbour).is_empty());
    }

    #[test]
    fn overlap_and_cover() {
        let other = Rect::new(30.0, 0.0, 100.0, 30.0);
        assert_eq!(BOX.intersect(other), Rect::new(30.0, 20.0, 40.0, 30.0));
        assert_eq!(BOX.union(other), Rect::new(10.0, 0.0, 100.0, 60.0));
        // A damaged area that is still empty must not pull the area to
        // repaint towards the corner it happens to sit in.
        let nothing = Rect::new(500.0, 500.0, 500.0, 900.0);
        assert_eq!((BOX.union(nothing), nothing.union(BOX)), (BOX, BOX));
        let stretched = BOX.including(Point::new(0.0, 10.0));
        assert_eq!(stretched, Rect::new(0.0, 10.0, 40.0, 60.0));
        let stretched = BOX.including(Point::new(50.0, 70.0));
        assert_eq!(stretched, Rect::new(10.0, 20.0, 50.0, 70.0));
        assert!(Rect::new(5.0, 5.0, 1.0, 9.0).is_empty() && !BOX.is_empty());
        assert!(Size::new(f64::NAN, 1.0).is_empty() && Size::new(1.0, 0.0).is_empty());
    }

    #[test]
    fn insets_move_each_edge_by_its_own_distance() {
        let padding = Insets::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(BOX.inset(padding), Rect::new(14.0, 21.0, 38.0, 57.0));
        assert_eq!(BOX.inset(padding).inset(-padding), BOX);
        assert_eq!(padding.size(), Size::new(6.0, 4.0));
        assert_eq!(Insets::symmetric(8.0, 2.0), Insets::new(2.0, 8.0, 2.0, 8.0));
    }

    #[test]
    fn rounding_out_covers_every_pixel_the_rectangle_touches() {
        let bounds = Rect::new(10.2, -3.7, 12.0, 4.01).round_out();
        assert_eq!(bounds, Rect::new(10.0, -4.0, 12.0, 5.0));
    }

    #[test]
    fn a_spring_moves_a_rectangle_edge_by_edge() {
        let grown = BOX.plus(Rect::new(-2.0, -2.0, 2.0, 2.0).scaled(2.0));
        assert_eq!(grown, Rect::new(6.0, 16.0, 44.0, 64.0));
        assert_eq!(
            Animatable::length_squared(Rect::new(1.0, 2.0, 3.0, 4.0)),
            30.0
        );
        assert_eq!(BOX.minus(Rect::zero()), BOX);
        let size = Size::new(3.0, 4.0);
        assert_eq!(size.plus(size.scaled(0.5)), Size::new(4.5, 6.0));
        assert_eq!(Animatable::length_squared(size.minus(Size::zero())), 25.0);
    }
}
