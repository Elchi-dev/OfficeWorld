//! Colour for `OfficeWorld`.
//!
//! [`Color`] is a colour the way files and screens give it: sRGB with an
//! alpha. It is what a document stores, what a gradient from a file is
//! blended in, and what contrast is measured on. [`Oklab`] is the same
//! colour the way the eye tells colours apart. It is what the interface
//! blends, animates and builds its palettes in.
//!
//! ```
//! use ow_color::{Color, Oklab};
//!
//! let brand = Color::from_hex("#1a73e8")?;
//! // The same hue, lighter, for a hover state. Asking for more lightness
//! // can ask for a colour no screen has, so it is brought back into range
//! // with its hue kept.
//! let mut lighter = Oklab::from(brand);
//! lighter.lightness += 0.15;
//! let hover = Color::from(lighter.to_gamut());
//! assert_eq!(hover.to_rgba8(), [106, 166, 255, 255]);
//!
//! // White text on the brand colour is readable by the guidelines' measure.
//! assert!(Color::WHITE.contrast(brand) >= 4.5);
//! # Ok::<(), ow_color::InvalidHex>(())
//! ```

mod oklab;
mod srgb;

pub use oklab::Oklab;
pub use srgb::{Color, InvalidHex};
