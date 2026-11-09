//! Rectangles with rounded corners.

use crate::point::Point;
use crate::rect::Rect;

/// The radii of the four corners of a rectangle, clockwise from the top
/// left, in the order CSS gives them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Radii {
    /// The top left corner.
    pub top_left: f64,
    /// The top right corner.
    pub top_right: f64,
    /// The bottom right corner.
    pub bottom_right: f64,
    /// The bottom left corner.
    pub bottom_left: f64,
}

impl Radii {
    /// Four square corners.
    pub const ZERO: Self = Self::uniform(0.0);

    /// The radii of the top left, top right, bottom right and bottom left
    /// corner.
    pub const fn new(top_left: f64, top_right: f64, bottom_right: f64, bottom_left: f64) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    /// The same radius on every corner.
    pub const fn uniform(radius: f64) -> Self {
        Self::new(radius, radius, radius, radius)
    }

    fn map(self, change: impl Fn(f64) -> f64) -> Self {
        Self::new(
            change(self.top_left),
            change(self.top_right),
            change(self.bottom_right),
            change(self.bottom_left),
        )
    }
}

/// A rectangle whose corners are quarter circles.
///
/// Its radii always fit: the two corners on a side never take more than the
/// side between them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RoundedRect {
    rect: Rect,
    radii: Radii,
}

impl RoundedRect {
    /// A rectangle with rounded corners.
    ///
    /// Radii too large for the rectangle are scaled down, all four by the
    /// same factor, until the corners on every side just meet. That is the
    /// rule CSS uses, so a radius of half the height on a wide button gives
    /// the same pill here as in a browser. A radius that is negative or not
    /// a number counts as zero.
    pub fn new(rect: Rect, radii: Radii) -> Self {
        let radii = radii.map(|radius| radius.max(0.0));
        let sides = [
            (rect.width(), radii.top_left + radii.top_right),
            (rect.height(), radii.top_right + radii.bottom_right),
            (rect.width(), radii.bottom_right + radii.bottom_left),
            (rect.height(), radii.bottom_left + radii.top_left),
        ];
        let fit = sides
            .iter()
            .filter(|(_, taken)| *taken > 0.0)
            .map(|(side, taken)| side.max(0.0) / taken)
            .fold(1.0, f64::min);
        Self {
            rect,
            radii: radii.map(|radius| radius * fit),
        }
    }

    /// The rectangle the corners are cut from.
    pub const fn rect(self) -> Rect {
        self.rect
    }

    /// The radii, after fitting them to the rectangle.
    pub const fn radii(self) -> Radii {
        self.radii
    }

    /// Whether a point is inside. The curve of a corner belongs to the
    /// shape.
    pub fn contains(self, point: Point) -> bool {
        let Rect { x0, y0, x1, y1 } = self.rect;
        let Radii {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        } = self.radii;
        // For each corner its radius, and how far the point lies past the
        // centre of the corner's circle, to the side and up or down.
        let corners = [
            (top_left, x0 + top_left - point.x, y0 + top_left - point.y),
            (
                top_right,
                point.x - (x1 - top_right),
                y0 + top_right - point.y,
            ),
            (
                bottom_right,
                point.x - (x1 - bottom_right),
                point.y - (y1 - bottom_right),
            ),
            (
                bottom_left,
                x0 + bottom_left - point.x,
                point.y - (y1 - bottom_left),
            ),
        ];
        // A point that is past the centre in one direction only is beside a
        // straight edge, and within the radius there as long as it is in
        // the rectangle. So the same test serves all of them.
        self.rect.contains(point)
            && corners
                .iter()
                .all(|&(radius, across, along)| across.max(0.0).hypot(along.max(0.0)) <= radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radii_that_do_not_fit_shrink_together() {
        // The example of the CSS specification: when the corners of a side
        // would overlap, every radius is scaled by the same factor. Here
        // the left side is 50 long and its corners ask for 40 + 40.
        let pill = RoundedRect::new(Rect::new(0.0, 0.0, 100.0, 50.0), Radii::uniform(40.0));
        assert_eq!(pill.radii(), Radii::uniform(25.0));
        let lopsided = RoundedRect::new(
            Rect::new(0.0, 0.0, 100.0, 50.0),
            Radii::new(90.0, 30.0, 0.0, 10.0),
        );
        assert_eq!(lopsided.radii(), Radii::new(45.0, 15.0, 0.0, 5.0));
        let fitting = Radii::new(10.0, 20.0, 30.0, 5.0);
        assert_eq!(
            RoundedRect::new(Rect::new(0.0, 0.0, 100.0, 50.0), fitting).radii(),
            fitting
        );

        let broken = RoundedRect::new(
            Rect::new(0.0, 0.0, 10.0, 10.0),
            Radii::new(-3.0, f64::NAN, 2.0, 0.0),
        );
        assert_eq!(broken.radii(), Radii::new(0.0, 0.0, 2.0, 0.0));
        let inside_out = RoundedRect::new(Rect::new(10.0, 0.0, 0.0, 10.0), Radii::uniform(4.0));
        assert_eq!(
            (inside_out.radii(), inside_out.rect().x0),
            (Radii::ZERO, 10.0)
        );
    }

    #[test]
    fn a_click_in_the_cut_off_corner_misses() {
        let radii = Radii::new(20.0, 10.0, 6.0, 20.0);
        let button = RoundedRect::new(Rect::new(0.0, 0.0, 100.0, 40.0), radii);
        // Each corner: a point in the square the circle is cut from, first
        // outside the circle, then inside it.
        assert!(!button.contains(Point::new(3.0, 3.0)) && button.contains(Point::new(7.0, 7.0)));
        assert!(!button.contains(Point::new(98.5, 1.5)) && button.contains(Point::new(96.0, 4.0)));
        assert!(
            !button.contains(Point::new(99.0, 39.0)) && button.contains(Point::new(97.0, 37.0))
        );
        assert!(!button.contains(Point::new(3.0, 37.0)) && button.contains(Point::new(7.0, 33.0)));
        // 12 and 16 past the centre of the first corner is exactly its
        // radius of 20 away: on the curve, which belongs to the button.
        assert!(button.contains(Point::new(8.0, 4.0)));
        // Away from the corners it is the plain rectangle.
        assert!(button.contains(Point::new(50.0, 0.0)) && !button.contains(Point::new(50.0, 40.0)));
        assert!(button.contains(Point::new(0.0, 20.0)) && !button.contains(Point::new(-1.0, 20.0)));
    }
}
