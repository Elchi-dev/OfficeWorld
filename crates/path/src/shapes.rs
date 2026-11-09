//! Paths of the common shapes.

use ow_geometry::{Point, Rect, RoundedRect, Vector};

use crate::path::Path;

impl Path {
    /// Straight lines from each point to the next, left open.
    pub fn polyline<P: Into<Point>>(points: impl IntoIterator<Item = P>) -> Self {
        let mut path = Self::new();
        for point in points {
            path.line_to(point);
        }
        path
    }

    /// The outline of a polygon: straight lines from each corner to the
    /// next, and back from the last to the first.
    pub fn polygon<P: Into<Point>>(corners: impl IntoIterator<Item = P>) -> Self {
        let mut path = Self::polyline(corners);
        path.close();
        path
    }

    /// The outline of a rectangle, clockwise on a screen from its top left
    /// corner.
    pub fn rect(rect: Rect) -> Self {
        let Rect { x0, y0, x1, y1 } = rect;
        Self::polygon([(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
    }

    /// The outline of a rectangle with rounded corners, clockwise on a
    /// screen from its top left corner.
    pub fn rounded_rect(rounded: RoundedRect) -> Self {
        let Rect { x0, y0, x1, y1 } = rounded.rect();
        let radii = rounded.radii();
        let (right, down) = (Vector::new(1.0, 0.0), Vector::new(0.0, 1.0));
        // Each corner with its radius, the direction the outline runs into
        // it and the one it leaves in.
        let corners = [
            (Point::new(x0, y0), radii.top_left, -down, right),
            (Point::new(x1, y0), radii.top_right, right, down),
            (Point::new(x1, y1), radii.bottom_right, down, -right),
            (Point::new(x0, y1), radii.bottom_left, -right, -down),
        ];
        let mut path = Self::new();
        for (corner, radius, into, out) in corners {
            path.line_to(corner - into * radius);
            path.arc_to(radius, radius, 0.0, false, true, corner + out * radius);
        }
        path.close();
        path
    }

    /// The outline of an ellipse around a centre, clockwise on a screen
    /// from its rightmost point. With both radii equal it is a circle.
    pub fn ellipse(centre: Point, radius_x: f64, radius_y: f64) -> Self {
        let right = Vector::new(radius_x, 0.0);
        let mut path = Self::new();
        path.move_to(centre + right);
        path.arc_to(radius_x, radius_y, 0.0, false, true, centre - right);
        path.arc_to(radius_x, radius_y, 0.0, false, true, centre + right);
        path.close();
        path
    }
}

#[cfg(test)]
mod tests {
    use ow_geometry::Radii;

    use super::*;

    #[test]
    fn a_circle_is_round_to_three_ten_thousandths() {
        let centre = Point::new(40.0, -20.0);
        let circle = Path::ellipse(centre, 50.0, 50.0);
        for point in &circle.flatten(1e-4)[0].points {
            let off = (point.distance(centre) / 50.0 - 1.0).abs();
            assert!(off < 2.8e-4, "{point:?} is {off} of the radius off");
        }
        // Its area is that of the circle to the same measure, and it runs
        // clockwise on a screen.
        let share = circle.area() / (std::f64::consts::PI * 2500.0);
        assert!((share - 1.0).abs() < 3e-4, "{share}");
        let oval = Path::ellipse(centre, 30.0, 10.0);
        assert_eq!(oval.bounds(), Some(Rect::new(10.0, -30.0, 70.0, -10.0)));
    }

    #[test]
    fn a_rounded_outline_has_the_same_inside_as_its_rounded_rectangle() {
        // Two crates answer the same question, one from the radii and one
        // from the outline. Every point of a grid has to get the same
        // answer from both. Only within a hundredth of the outline may they
        // differ: that is how far its curves stray from true circles.
        let rect = Rect::new(10.0, 10.0, 90.0, 60.0);
        let rounded = RoundedRect::new(rect, Radii::new(12.0, 0.0, 25.0, 6.0));
        let path = Path::rounded_rect(rounded);
        let outline = &path.flatten(1e-3)[0];
        for x in 0..=200 {
            for y in 0..=140 {
                let point = Point::new(f64::from(x) / 2.0 + 0.123, f64::from(y) / 2.0 + 0.071);
                let agree = (outline.winding(point) != 0) == rounded.contains(point);
                assert!(agree || outline.distance_to(point) < 0.01, "{point:?}");
            }
        }
        // A square corner is a corner, not a curve without length, and
        // without any radius the outline is that of the rectangle.
        assert_eq!(path.to_string().matches('C').count(), 3);
        let square = RoundedRect::new(rect, Radii::uniform(0.0));
        assert_eq!(Path::rounded_rect(square), Path::rect(rect));
    }

    #[test]
    fn lines_run_from_point_to_point() {
        let rect = Path::rect(Rect::new(1.0, 2.0, 6.0, 4.0));
        assert_eq!(rect.to_string(), "M1 2 L6 2 L6 4 L1 4 Z");
        let open = Path::polyline([(1.0, 1.0), (5.0, 4.0), (2.0, 2.0)]);
        assert_eq!(open.to_string(), "M1 1 L5 4 L2 2");
    }
}
