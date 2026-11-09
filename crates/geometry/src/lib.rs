//! Geometry for `OfficeWorld`.
//!
//! The shapes every other part measures with. Coordinates are `f64`, x to
//! the right and y downwards, as on a screen and on a page. A sheet can be a
//! million rows tall, and at that distance from the origin an `f32` no
//! longer tells two neighbouring pixels apart.

mod point;
mod rect;

pub use point::{Point, Vector};
pub use rect::{Insets, Rect, Size};
