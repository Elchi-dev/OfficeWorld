//! Affine transforms: moving, scaling, turning and shearing the plane.

#![expect(
    clippy::many_single_char_names,
    reason = "the six coefficients keep the names CSS, SVG and PDF give them"
)]

use std::ops::Mul;

use crate::point::{Point, Vector};
use crate::rect::Rect;

/// A transform that keeps straight lines straight and parallel lines
/// parallel.
///
/// It is the matrix CSS, SVG and PDF write as `matrix(a, b, c, d, e, f)`: a
/// point `(x, y)` goes to `(a * x + c * y + e, b * x + d * y + f)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine([f64; 6]);

impl Affine {
    /// The transform that leaves everything where it is.
    pub const IDENTITY: Self = Self::scale(1.0, 1.0);

    /// A transform from its six coefficients, in the order CSS, SVG and PDF
    /// give them.
    pub const fn new(coefficients: [f64; 6]) -> Self {
        Self(coefficients)
    }

    /// The six coefficients.
    pub const fn coefficients(self) -> [f64; 6] {
        self.0
    }

    /// Moves everything by an offset.
    pub const fn translate(offset: Vector) -> Self {
        Self([1.0, 0.0, 0.0, 1.0, offset.x, offset.y])
    }

    /// Stretches everything away from the origin, by one factor to the
    /// right and one downwards.
    pub const fn scale(x: f64, y: f64) -> Self {
        Self([x, 0.0, 0.0, y, 0.0, 0.0])
    }

    /// Turns everything around the origin by an angle in radians. On a
    /// screen, where y grows downwards, a positive angle is clockwise.
    pub fn rotate(radians: f64) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self([cos, sin, -sin, cos, 0.0, 0.0])
    }

    /// Slants everything: x grows with y by the tangent of the first angle,
    /// y with x by the tangent of the second. Both are in radians.
    pub fn skew(x_radians: f64, y_radians: f64) -> Self {
        Self([1.0, y_radians.tan(), x_radians.tan(), 1.0, 0.0, 0.0])
    }

    /// The same transform, done around a point other than the origin.
    #[must_use]
    pub fn around(self, centre: Point) -> Self {
        Self::translate(centre.to_vector()) * self * Self::translate(-centre.to_vector())
    }

    /// The factor by which the transform changes areas. It is negative when
    /// the transform mirrors, and zero when it flattens the plane to a line.
    pub fn determinant(self) -> f64 {
        let [a, b, c, d, _, _] = self.0;
        a * d - b * c
    }

    /// The transform that undoes this one, or `None` when this one flattens
    /// the plane and cannot be undone.
    pub fn inverse(self) -> Option<Self> {
        let [a, b, c, d, e, f] = self.0;
        let determinant = self.determinant();
        (determinant != 0.0 && determinant.is_finite()).then(|| {
            Self([
                d / determinant,
                -b / determinant,
                -c / determinant,
                a / determinant,
                (c * f - d * e) / determinant,
                (b * e - a * f) / determinant,
            ])
        })
    }

    /// Whether the transform only moves things: it neither scales, turns
    /// nor shears, so a rectangle stays the same rectangle elsewhere.
    pub fn is_translation(self) -> bool {
        let [a, b, c, d, _, _] = self.0;
        (a, b, c, d) == (1.0, 0.0, 0.0, 1.0)
    }

    /// The smallest rectangle along the axes that covers a rectangle after
    /// the transform.
    pub fn bounds_of(self, rect: Rect) -> Rect {
        let corners = [
            self * Point::new(rect.x0, rect.y0),
            self * Point::new(rect.x1, rect.y0),
            self * Point::new(rect.x1, rect.y1),
            self * Point::new(rect.x0, rect.y1),
        ];
        let lowest = |values: [f64; 4]| values.into_iter().fold(f64::INFINITY, f64::min);
        let highest = |values: [f64; 4]| values.into_iter().fold(f64::NEG_INFINITY, f64::max);
        Rect::new(
            lowest(corners.map(|corner| corner.x)),
            lowest(corners.map(|corner| corner.y)),
            highest(corners.map(|corner| corner.x)),
            highest(corners.map(|corner| corner.y)),
        )
    }
}

impl Default for Affine {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// `first * second` does `second` and then `first`, as matrices multiply.
impl Mul for Affine {
    type Output = Self;

    fn mul(self, inner: Self) -> Self {
        let [a, b, c, d, e, f] = self.0;
        let [g, h, i, j, k, l] = inner.0;
        Self([
            a * g + c * h,
            b * g + d * h,
            a * i + c * j,
            b * i + d * j,
            a * k + c * l + e,
            b * k + d * l + f,
        ])
    }
}

impl Mul<Point> for Affine {
    type Output = Point;

    fn mul(self, point: Point) -> Point {
        let [a, b, c, d, e, f] = self.0;
        Point::new(a * point.x + c * point.y + e, b * point.x + d * point.y + f)
    }
}

/// A vector is a difference of points, so moving the plane does not change
/// it: only the scaling, turning and shearing part applies.
impl Mul<Vector> for Affine {
    type Output = Vector;

    fn mul(self, vector: Vector) -> Vector {
        let [a, b, c, d, _, _] = self.0;
        Vector::new(a * vector.x + c * vector.y, b * vector.x + d * vector.y)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

    use super::*;

    fn close(a: Point, b: Point) -> bool {
        a.distance(b) < 1e-12
    }

    #[test]
    fn the_order_of_two_transforms_matters() {
        let move_it = Affine::translate(Vector::new(10.0, 0.0));
        let double_it = Affine::scale(2.0, 3.0);
        let place = Point::new(1.0, 1.0);
        // The right factor is done first.
        assert_eq!((move_it * double_it) * place, Point::new(12.0, 3.0));
        assert_eq!((double_it * move_it) * place, Point::new(22.0, 3.0));
        assert_eq!(move_it * (double_it * place), (move_it * double_it) * place);
        assert_eq!(Affine::default() * place, place);
        assert_eq!(
            (move_it * double_it).coefficients(),
            [2.0, 0.0, 0.0, 3.0, 10.0, 0.0]
        );
        // Two transforms without a zero or a one in them, multiplied by
        // hand.
        let product = Affine::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
            * Affine::new([7.0, 8.0, 9.0, 10.0, 11.0, 12.0]);
        assert_eq!(product.coefficients(), [31.0, 46.0, 39.0, 58.0, 52.0, 76.0]);
        assert_eq!(
            Affine::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]) * place,
            Point::new(9.0, 12.0)
        );
    }

    #[test]
    fn turning_is_clockwise_on_a_screen() {
        let quarter = Affine::rotate(FRAC_PI_2);
        assert!(close(quarter * Point::new(1.0, 0.0), Point::new(0.0, 1.0)));
        assert!(close(quarter * Point::new(0.0, 1.0), Point::new(-1.0, 0.0)));
        // Around a centre, the centre stays and the rest turns about it.
        let about = quarter.around(Point::new(5.0, 5.0));
        assert!(close(about * Point::new(5.0, 5.0), Point::new(5.0, 5.0)));
        assert!(close(about * Point::new(6.0, 5.0), Point::new(5.0, 6.0)));
    }

    #[test]
    fn a_shear_slants_one_axis_along_the_other() {
        let slanted = Affine::skew(FRAC_PI_4, 0.0) * Point::new(0.0, 2.0);
        assert!(close(slanted, Point::new(2.0, 2.0)));
        let lifted = Affine::skew(0.0, FRAC_PI_4) * Point::new(3.0, 0.0);
        assert!(close(lifted, Point::new(3.0, 3.0)));
    }

    #[test]
    fn a_vector_is_not_moved_by_a_translation() {
        let transform = Affine::translate(Vector::new(100.0, 100.0)) * Affine::scale(2.0, -1.0);
        assert_eq!(transform * Vector::new(3.0, 4.0), Vector::new(6.0, -4.0));
        let general = Affine::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(general * Vector::new(2.0, 3.0), Vector::new(11.0, 16.0));
    }

    #[test]
    fn the_inverse_undoes_the_transform() {
        let transform = Affine::translate(Vector::new(7.0, -2.0))
            * Affine::rotate(0.6)
            * Affine::skew(0.2, -0.1)
            * Affine::scale(3.0, 0.5);
        let back = transform.inverse().unwrap();
        for place in [
            Point::new(0.0, 0.0),
            Point::new(13.0, -4.5),
            Point::new(-200.0, 77.0),
        ] {
            assert!(close(back * (transform * place), place), "{place:?}");
        }
        assert!((transform.determinant() * back.determinant() - 1.0).abs() < 1e-12);
        assert_eq!(
            transform
                .inverse()
                .unwrap()
                .inverse()
                .map(Affine::is_translation),
            Some(false)
        );

        // A scale of zero flattens the plane onto a line. Nothing can bring
        // the lost direction back, and dividing by the zero would hand out
        // infinities.
        assert_eq!(Affine::scale(2.0, 0.0).inverse(), None);
        assert_eq!(Affine::scale(f64::INFINITY, 1.0).inverse(), None);
        assert_eq!(Affine::scale(2.0, -3.0).determinant(), -6.0);
    }

    #[test]
    fn only_a_translation_keeps_a_rectangle_as_it_is() {
        assert!(Affine::translate(Vector::new(3.0, 4.0)).is_translation());
        for other in [
            Affine::scale(2.0, 1.0),
            Affine::scale(1.0, 2.0),
            Affine::rotate(0.1),
            Affine::skew(0.1, 0.0),
        ] {
            assert!(!other.is_translation(), "{other:?}");
        }
    }

    #[test]
    fn bounds_cover_a_turned_rectangle() {
        // A square of side 2 around the origin, turned an eighth, stands on
        // its corner: its bounds reach out to its half diagonal.
        let square = Rect::new(-1.0, -1.0, 1.0, 1.0);
        let bounds = Affine::rotate(FRAC_PI_4).bounds_of(square);
        let reach = 2.0_f64.sqrt();
        assert!(close(bounds.origin(), Point::new(-reach, -reach)));
        assert!(close(
            Point::new(bounds.x1, bounds.y1),
            Point::new(reach, reach)
        ));
        let moved =
            Affine::translate(Vector::new(5.0, 6.0)).bounds_of(Rect::new(0.0, 1.0, 2.0, 4.0));
        assert_eq!(moved, Rect::new(5.0, 7.0, 7.0, 10.0));
        let mirrored = Affine::scale(-1.0, 2.0).bounds_of(Rect::new(1.0, 1.0, 3.0, 2.0));
        assert_eq!(mirrored, Rect::new(-3.0, 2.0, -1.0, 4.0));
    }
}
