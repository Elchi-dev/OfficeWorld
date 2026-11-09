//! The design system of `OfficeWorld`.
//!
//! What every part of the interface looks like and how it moves is decided
//! here once: the colours, the sizes and the springs. A part asks for "the
//! soft text colour of the accent" or "the quick spring" and never for a
//! number of its own, so the interface stays one thing, and can be changed
//! as one thing.
//!
//! Colours are grown from a [`Tint`] into a [`Ramp`], worked out in Oklab
//! so that every hue has the same steps and so that text can be read:
//!
//! ```
//! use ow_design::{Ramp, Scheme, Tint};
//!
//! let blue = Ramp::new(Tint::BLUE, Scheme::Light);
//! // A filled button: white on blue, at the contrast the guidelines ask.
//! assert_eq!(blue.solid.to_rgba8(), [31, 115, 229, 255]);
//! assert!(blue.on_solid.contrast(blue.solid) >= 4.5);
//!
//! // The same tint on a dark scheme keeps its solid colour and turns the
//! // rest around.
//! let night = Ramp::new(Tint::BLUE, Scheme::Dark);
//! assert_eq!(night.solid, blue.solid);
//! assert!(night.text.luminance() > night.backdrop.luminance());
//! ```
//!
//! Not built: the choice of fonts, icons, shadows, and the styles of the
//! single controls.

pub mod metrics;
mod motion;
mod ramp;

pub use motion::Motion;
pub use ramp::{Ramp, Scheme, Tint};
