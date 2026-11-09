//! Colour for `OfficeWorld`.
//!
//! [`Color`] is a colour the way files and screens give it: sRGB with an
//! alpha. It is what a document stores, what a gradient from a file is
//! blended in, and what contrast is measured on.

mod srgb;

pub use srgb::{Color, InvalidHex};
