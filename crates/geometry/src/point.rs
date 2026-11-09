//! Positions and the differences between them.

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use ow_motion::Animatable;

/// A position.
///
/// A point is not a vector: two positions cannot be added, and the
/// difference of two positions is a [`Vector`]. Keeping them apart lets the
/// compiler catch an offset used as a place.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// The distance from the origin to the right.
    pub x: f64,
    /// The distance from the origin downwards.
    pub y: f64,
}

/// A direction with a length: the way from one point to another.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector {
    /// The part to the right.
    pub x: f64,
    /// The part downwards.
    pub y: f64,
}

impl Point {
    /// The point every coordinate is measured from.
    pub const ORIGIN: Self = Self::new(0.0, 0.0);

    /// A point from its coordinates.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// The way from the origin to this point.
    pub const fn to_vector(self) -> Vector {
        Vector::new(self.x, self.y)
    }

    /// How far it is to another point.
    pub fn distance(self, other: Self) -> f64 {
        (other - self).length()
    }

    /// The point a share `t` of the way to another: this point at 0, the
    /// other at 1.
    #[must_use]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

impl Vector {
    /// The vector that leads nowhere.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// A vector from its parts.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// The point this vector leads to from the origin.
    pub const fn to_point(self) -> Point {
        Point::new(self.x, self.y)
    }

    /// The length.
    pub fn length(self) -> f64 {
        self.x.hypot(self.y)
    }

    /// The length squared, which needs no square root.
    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    /// The dot product: the lengths multiplied, times the cosine of the
    /// angle between the two.
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// The cross product: the lengths multiplied, times the sine of the
    /// angle from this vector to the other. It is positive when the other
    /// lies clockwise of this one on a screen, where y grows downwards.
    pub fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// The vector of length 1 in the same direction, or `None` for a vector
    /// too short to have one.
    pub fn normalized(self) -> Option<Self> {
        let length = self.length();
        (length > 0.0 && length.is_finite()).then(|| self / length)
    }

    /// The vector turned a quarter clockwise on a screen.
    #[must_use]
    pub const fn perpendicular(self) -> Self {
        Self::new(-self.y, self.x)
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, other: Self) -> Vector {
        Vector::new(self.x - other.x, self.y - other.y)
    }
}

impl Add<Vector> for Point {
    type Output = Self;

    fn add(self, offset: Vector) -> Self {
        Self::new(self.x + offset.x, self.y + offset.y)
    }
}

impl Sub<Vector> for Point {
    type Output = Self;

    fn sub(self, offset: Vector) -> Self {
        Self::new(self.x - offset.x, self.y - offset.y)
    }
}

impl AddAssign<Vector> for Point {
    fn add_assign(&mut self, offset: Vector) {
        *self = *self + offset;
    }
}

impl SubAssign<Vector> for Point {
    fn sub_assign(&mut self, offset: Vector) {
        *self = *self - offset;
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl Mul<f64> for Vector {
    type Output = Self;

    fn mul(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
}

impl Div<f64> for Vector {
    type Output = Self;

    fn div(self, divisor: f64) -> Self {
        Self::new(self.x / divisor, self.y / divisor)
    }
}

impl Animatable for Vector {
    fn zero() -> Self {
        Self::ZERO
    }

    fn plus(self, other: Self) -> Self {
        self + other
    }

    fn scaled(self, factor: f64) -> Self {
        self * factor
    }

    fn length_squared(self) -> f64 {
        self.length_squared()
    }
}

// A point is animated as the vector from the origin. Adding two points has
// no meaning in geometry, and a spring needs it all the same: it moves a
// value by adding scaled copies of where it was and how fast it went.
impl Animatable for Point {
    fn zero() -> Self {
        Self::ORIGIN
    }

    fn plus(self, other: Self) -> Self {
        self + other.to_vector()
    }

    fn scaled(self, factor: f64) -> Self {
        (self.to_vector() * factor).to_point()
    }

    fn length_squared(self) -> f64 {
        self.to_vector().length_squared()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_and_vectors_keep_their_arithmetic_apart() {
        let (a, b) = (Point::new(1.0, 2.0), Point::new(4.0, 6.0));
        let way = b - a;
        assert_eq!(way, Vector::new(3.0, 4.0));
        assert_eq!((a + way, b - way), (b, a));
        assert_eq!(
            (way.length(), way.length_squared(), a.distance(b)),
            (5.0, 25.0, 5.0)
        );
        assert_eq!(a.lerp(b, 0.25), Point::new(1.75, 3.0));
        assert_eq!(way + way - way * 3.0, -way);
        assert_eq!((way / 2.0).to_point(), Point::new(1.5, 2.0));

        let mut moved = a;
        moved += way;
        moved -= Vector::new(1.0, 1.0);
        assert_eq!(moved, Point::new(3.0, 5.0));
    }

    #[test]
    fn products_say_how_two_directions_lie() {
        // On a screen y grows downwards, so a quarter turn clockwise takes
        // "right" to "down", and the cross product of the two is positive.
        let (right, down) = (Vector::new(2.0, 0.0), Vector::new(0.0, 3.0));
        assert_eq!(
            (right.dot(down), right.cross(down), down.cross(right)),
            (0.0, 6.0, -6.0)
        );
        assert_eq!(
            Vector::new(3.0, 4.0).perpendicular(),
            Vector::new(-4.0, 3.0)
        );
        assert_eq!(Vector::new(3.0, 4.0).dot(Vector::new(2.0, 1.0)), 10.0);
    }

    #[test]
    fn a_direction_needs_a_length() {
        assert_eq!(
            Vector::new(0.0, -4.0).normalized(),
            Some(Vector::new(0.0, -1.0))
        );
        // Dividing by a length of zero would hand back a direction made of
        // not-a-numbers, which then spreads through every transform.
        assert_eq!(Vector::ZERO.normalized(), None);
        assert_eq!(Vector::new(f64::INFINITY, 0.0).normalized(), None);
    }

    #[test]
    fn a_spring_moves_a_point_as_it_moves_its_coordinates() {
        let (place, speed) = (Point::new(3.0, -1.0), Point::new(0.0, 40.0));
        assert_eq!(place.plus(speed.scaled(0.5)), Point::new(3.0, 19.0));
        assert_eq!(place.minus(Point::zero()), place);
        assert_eq!(Animatable::length_squared(place), 10.0);
        let way = Vector::new(3.0, 4.0);
        assert_eq!(way.plus(way.scaled(-2.0)), Vector::new(-3.0, -4.0));
        assert_eq!(Animatable::length_squared(way.minus(Vector::zero())), 25.0);
    }
}
