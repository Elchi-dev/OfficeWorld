//! Coverage: how much of each pixel a shape covers.

use ow_geometry::{Point, Rect, Vector};
use ow_path::{FillRule, Path, Polyline};

/// How far the straight lines a curve is drawn with may stray from it, in
/// pixels. A tenth of a pixel cannot be told from the curve.
pub(crate) const TOLERANCE: f64 = 0.1;

/// How much of each pixel of a picture a shape covers: nothing, all of it,
/// or the part of it that lies inside the shape's edge.
///
/// The coverage of a pixel is the exact area of the shape within it, not a
/// count of sample points, so an edge is as smooth as the pixels allow at
/// any angle. It is computed the way font rasterizers do it. Each edge
/// adds its part to the pixels it passes, the parts of a row are summed
/// from left to right, and the fill rule is applied to the sum. Where two
/// edges of a shape that overlaps itself cross inside one pixel, that pixel
/// can come out a little too full or too empty. Everywhere else the
/// coverage is exact.
#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    left: u32,
    top: u32,
    width: u32,
    coverage: Vec<f32>,
}

impl Mask {
    /// The coverage of a path filled by a rule, in a picture of a size.
    ///
    /// One unit of the path is one pixel, and the pixel in the top left
    /// corner covers the square from (0, 0) to (1, 1). The mask only holds
    /// the rows and columns the path reaches, so a small shape in a large
    /// picture is cheap. A path with a point that is not a number or
    /// infinite covers nothing.
    pub fn new(path: &Path, rule: FillRule, width: u32, height: u32) -> Self {
        let picture = Rect::new(0.0, 0.0, f64::from(width), f64::from(height));
        let pieces = path.flatten(TOLERANCE);
        let mut corners = pieces.iter().flat_map(|piece| &piece.points);
        let finite = corners.all(|corner| corner.x.is_finite() && corner.y.is_finite());
        let reach = path.bounds().filter(|_| finite).unwrap_or_default();
        let reach = reach.intersect(picture).round_out();
        let (left, top) = (whole(reach.x0), whole(reach.y0));
        let mut cells = Cells::new(whole(reach.width()), whole(reach.height()));
        let offset = Vector::new(reach.x0, reach.y0);
        for (from, to) in pieces.iter().flat_map(Polyline::edges) {
            cells.edge(from - offset, to - offset);
        }
        Self {
            left,
            top,
            width: cells.width,
            coverage: cells.coverage(rule),
        }
    }

    /// How much of a pixel is covered, from 0 for none of it to 1 for all
    /// of it.
    pub fn coverage(&self, x: u32, y: u32) -> f32 {
        let within = |at: u32, start: u32| at.checked_sub(start).map(|at| at as usize);
        let cell = within(x, self.left)
            .filter(|column| *column < self.width as usize)
            .zip(within(y, self.top))
            .and_then(|(column, row)| self.coverage.get(row * self.width as usize + column));
        cell.copied().unwrap_or(0.0)
    }

    /// The rows the mask holds, from the top: the column and the row each
    /// one begins at in the picture, and the coverage of its pixels from
    /// there to the right.
    pub fn rows(&self) -> impl Iterator<Item = (u32, u32, &[f32])> {
        let rows = self.coverage.chunks(self.width.max(1) as usize);
        (self.top..).zip(rows).map(|(y, row)| (self.left, y, row))
    }
}

/// A whole number of pixels from a position within the picture.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "positions are kept between zero and the size of the picture, which is a u32"
)]
fn whole(position: f64) -> u32 {
    position as u32
}

/// The area under `min(u, 1)` from zero up to a point: how much of a column
/// one pixel wide lies right of an edge that slants across it, added up
/// along the edge.
fn ramp(reach: f64) -> f64 {
    let rising = reach.clamp(0.0, 1.0);
    rising * rising / 2.0 + (reach - 1.0).max(0.0)
}

/// The pixels of a mask while the edges of a shape are being added.
///
/// A cell holds how much the coverage of its pixel differs from that of
/// the pixel on its left. An edge then only touches the cells it passes
/// through, however wide the shape is. Each row has two cells more than
/// pixels, for what lies on and beyond the right side of the mask.
struct Cells {
    width: u32,
    height: u32,
    deltas: Vec<f64>,
}

impl Cells {
    fn new(width: u32, height: u32) -> Self {
        let deltas = vec![0.0; (width as usize + 2) * height as usize];
        Self {
            width,
            height,
            deltas,
        }
    }

    /// Adds an edge of a shape, given from where the mask begins.
    fn edge(&mut self, from: Point, to: Point) {
        // An edge that runs along the rows passes through none of them.
        let slope = (to.x - from.x) / (to.y - from.y);
        if !slope.is_finite() {
            return;
        }
        // An edge that runs down the picture counts one way, and one that
        // runs up it the other.
        let sign = (to.y - from.y).signum();
        // Left of the mask, an edge still counts for everything to its
        // right, so it is pulled onto the mask's left side. Right of the
        // mask it counts for nothing inside, and is pulled onto the right
        // side. Where the edge crosses a side it gets a bend, and between
        // two bends it is straight.
        let (width, height) = (f64::from(self.width), f64::from(self.height));
        let start = from.y.min(to.y).clamp(0.0, height);
        let end = from.y.max(to.y).clamp(0.0, height);
        let x_at = |y: f64| (from.x + (y - from.y) * slope).clamp(0.0, width);
        let bend = |side: f64| (from.y + (side - from.x) / slope).max(start).min(end);
        let (first, second) = (bend(0.0).min(bend(width)), bend(0.0).max(bend(width)));
        for (above, below) in [(start, first), (first, second), (second, end)] {
            let (upper, lower) = (
                Point::new(x_at(above), above),
                Point::new(x_at(below), below),
            );
            self.straight(upper, lower, sign);
        }
    }

    /// Adds a straight part of an edge that lies within the mask, given
    /// from its upper to its lower end.
    fn straight(&mut self, top: Point, bottom: Point, sign: f64) {
        if top.y >= bottom.y {
            return;
        }
        let slope = (bottom.x - top.x) / (bottom.y - top.y);
        let (width, stride) = (f64::from(self.width), self.width as usize + 2);
        for row in whole(top.y.floor())..whole(bottom.y.ceil()) {
            // The part of the edge in this row: how much of the row's
            // height it passes, and between which two places.
            let (above, below) = (
                top.y.max(f64::from(row)),
                bottom.y.min(f64::from(row) + 1.0),
            );
            let x_at = |y: f64| (top.x + (y - top.y) * slope).clamp(0.0, width);
            let (left, right) = (x_at(above).min(x_at(below)), x_at(above).max(x_at(below)));
            let passed = sign * (below - above);
            let (first, last) = (whole(left.floor()), whole(right.floor()));
            let cells = &mut self.deltas[row as usize * stride..];
            // The share of each column that lies right of the edge. Left of
            // the edge it is nothing, right of it everything, and a column
            // the edge slants across has the mean in between.
            let mut before = 0.0;
            for column in first..=last {
                let past = f64::from(column) + 1.0 - left;
                let share = if first == last {
                    f64::from(column) + 1.0 - f64::midpoint(left, right)
                } else {
                    (ramp(past) - ramp(past - (right - left))) / (right - left)
                };
                cells[column as usize] += passed * (share - before);
                before = share;
            }
            cells[last as usize + 1] += passed * (1.0 - before);
        }
    }

    /// The coverage of each pixel: the cells of a row summed from the left
    /// give how often the shape winds around each pixel on average, and the
    /// rule says how much of the pixel that fills.
    fn coverage(&self, rule: FillRule) -> Vec<f32> {
        let rows = self.deltas.chunks(self.width as usize + 2);
        let filled = move |winding: f64| match rule {
            FillRule::NonZero => winding.abs().min(1.0),
            FillRule::EvenOdd => {
                let folded = winding.abs() % 2.0;
                folded.min(2.0 - folded)
            }
        };
        let row = |cells: &[f64]| {
            let pixels = cells[..self.width as usize].iter();
            let sums = pixels.scan(0.0, |sum, cell| {
                *sum += cell;
                Some(*sum)
            });
            sums.map(filled).map(narrow).collect::<Vec<f32>>()
        };
        rows.flat_map(row).collect()
    }
}

/// A coverage as the `f32` a mask stores.
#[expect(
    clippy::cast_possible_truncation,
    reason = "a coverage lies between 0 and 1, where an f32 is fine enough"
)]
fn narrow(coverage: f64) -> f32 {
    coverage as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The part of a shape with straight sides for which `inside` is not
    /// negative: what is left of it when it is cut along a straight line.
    fn cut(shape: &[Point], inside: impl Fn(Point) -> f64) -> Vec<Point> {
        let mut kept = Vec::new();
        for (index, &to) in shape.iter().enumerate() {
            let from = shape[(index + shape.len() - 1) % shape.len()];
            let (before, after) = (inside(from), inside(to));
            if (before >= 0.0) != (after >= 0.0) {
                kept.push(from.lerp(to, before / (before - after)));
            }
            if after >= 0.0 {
                kept.push(to);
            }
        }
        kept
    }

    /// The area of the part of a triangle that lies in the pixel at
    /// (x, y), counted negative for a triangle that runs against the
    /// clock.
    fn area_in_pixel(triangle: [Point; 3], x: u32, y: u32) -> f64 {
        let (x, y) = (f64::from(x), f64::from(y));
        let shape = cut(&triangle, |point| point.x - x);
        let shape = cut(&shape, |point| x + 1.0 - point.x);
        let shape = cut(&shape, |point| point.y - y);
        Path::polygon(cut(&shape, |point| y + 1.0 - point.y)).area()
    }

    #[test]
    fn coverage_is_the_area_of_each_pixel_that_the_shape_winds_around() {
        // A star that crosses itself and reaches past every side of the
        // picture, a sliver thinner than a pixel, and a shape with edges
        // along the pixel grid, across it and nearly along it. For every
        // pixel, the mask is compared with the shape cut into triangles
        // and each triangle cut to the pixel.
        let shapes = [
            "M12 -3 L20.5 19 L-2 4.5 L26 5.25 L3.25 18.5 Z",
            "M1.2 1.3 L22.7 14.1 L22.9 14.6 Z",
            "M2 2 H9.5 V7.25 L9.5001 12 L4 12.5 L2 7 Z",
        ];
        for data in shapes {
            let path: Path = data.parse().unwrap();
            let corners = &path.flatten(TOLERANCE)[0].points;
            for rule in [FillRule::NonZero, FillRule::EvenOdd] {
                let mask = Mask::new(&path, rule, 24, 16);
                for (x, y) in (0..24).flat_map(|x| (0..16).map(move |y| (x, y))) {
                    let part = |pair: &[Point]| area_in_pixel([corners[0], pair[0], pair[1]], x, y);
                    let winding = corners.windows(2).map(part).sum::<f64>().abs();
                    let expected = match rule {
                        FillRule::NonZero => winding.min(1.0),
                        FillRule::EvenOdd => 1.0 - (winding % 2.0 - 1.0).abs(),
                    };
                    let off = (f64::from(mask.coverage(x, y)) - expected).abs();
                    assert!(off < 1e-6, "{data} {rule:?} at ({x}, {y}): {off}");
                }
            }
        }
    }

    #[test]
    fn a_mask_holds_only_the_pixels_its_shape_reaches() {
        // A diamond 10 across in a picture a thousand pixels wide: 10 rows
        // of 10 pixels, which together hold its area of 50.
        let diamond: Path = "M100 195 L105 200 L100 205 L95 200 Z".parse().unwrap();
        let mask = Mask::new(&diamond, FillRule::NonZero, 1000, 1000);
        let rows: Vec<_> = mask.rows().collect();
        let held = (rows.len(), rows[0].0, rows[0].1, rows[0].2.len());
        assert_eq!(held, (10, 95, 195, 10));
        let area: f32 = rows.iter().flat_map(|row| row.2).sum();
        assert!((area - 50.0).abs() < 1e-4, "{area}");
        // Beside the rows and columns it holds, nothing is covered.
        let at = |x, y| mask.coverage(x, y);
        assert_eq!((at(100, 200), at(94, 200)), (1.0, 0.0));
        assert_eq!((at(100, 205), at(105, 199)), (0.0, 0.0));
        // A shape outside the picture, one with a corner that is nowhere,
        // one that never ends and no shape at all cover nothing.
        let outside: Path = "M-9 -9 H-2 V-2 Z".parse().unwrap();
        let nowhere = Path::polygon([(1.0, 1.0), (9.0, 1.0), (5.0, f64::NAN)]);
        let endless = Path::polygon([(1.0, 1.0), (9.0, 1.0), (5.0, f64::INFINITY)]);
        for nothing in [outside, nowhere, endless, Path::new()] {
            let mask = Mask::new(&nothing, FillRule::NonZero, 20, 20);
            assert_eq!(mask.rows().count(), 0, "{nothing}");
        }
    }
}
