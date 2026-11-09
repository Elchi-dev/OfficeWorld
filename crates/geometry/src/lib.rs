//! Geometry for `OfficeWorld`.
//!
//! The shapes every other part measures with: [`Point`] and [`Vector`],
//! [`Size`], [`Rect`] and [`Insets`], [`RoundedRect`] and the [`Affine`]
//! transform. Coordinates are `f64`, x to the right and y downwards, as on a
//! screen and on a page. A sheet can be a million rows tall, and at that
//! distance from the origin an `f32` no longer tells two neighbouring pixels
//! apart.
//!
//! ```
//! use ow_geometry::{Affine, Point, Rect, Vector};
//!
//! let page = Rect::new(0.0, 0.0, 595.0, 842.0);
//! // Zoomed to 150 % and scrolled 300 down.
//! let view = Affine::translate(Vector::new(0.0, -300.0)) * Affine::scale(1.5, 1.5);
//! assert_eq!(view * Point::new(100.0, 400.0), Point::new(150.0, 300.0));
//! assert_eq!(view.bounds_of(page), Rect::new(0.0, -300.0, 892.5, 963.0));
//!
//! // A click on the screen, taken back to the page.
//! let back = view.inverse().expect("a zoom can be undone");
//! assert!(page.contains(back * Point::new(150.0, 300.0)));
//! ```

mod affine;
mod point;
mod rect;
mod rounded;

pub use affine::Affine;
pub use point::{Point, Vector};
pub use rect::{Insets, Rect, Size};
pub use rounded::{Radii, RoundedRect};
