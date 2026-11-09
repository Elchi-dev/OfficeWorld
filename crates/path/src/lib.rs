//! Paths for `OfficeWorld`.
//!
//! A [`Path`] is an outline of straight lines and curves: the shape of a
//! letter, a chart's line, the border of a cell. This crate builds paths,
//! measures them and turns them into the simpler shapes that drawing needs.
//! It draws nothing itself.

mod arc;
mod flatten;
mod measure;
mod path;
mod svg;

pub use flatten::Polyline;
pub use measure::FillRule;
pub use path::{Element, Path, Piece, Segment};
pub use svg::InvalidPathData;
