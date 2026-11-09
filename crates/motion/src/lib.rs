//! Motion for `OfficeWorld`.
//!
//! Everything that moves on screen gets its position from this crate. It
//! draws nothing and keeps no clock: it answers where a value is at a time
//! the caller names, in seconds.
//!
//! [`Spring`] is a damped spring solved in closed form. Its answer depends
//! only on the time asked for, not on how many frames were drawn before, so
//! an animation looks the same at any refresh rate. [`Animated`] is a value
//! such a spring moves, and its target can change mid-flight without a jump.
//!
//! ```
//! use ow_motion::{Animated, Spring};
//!
//! let mut width = Animated::new(240.0, Spring::from_response(0.3, 1.0)?);
//! width.set_target(320.0, 10.0);
//! // Half way open, the panel is told to close again.
//! width.set_target(240.0, 10.1);
//! assert!(width.value(10.1) > 240.0 && width.value(10.1) < 320.0);
//! assert!(width.is_settled(12.0, 0.01));
//! # Ok::<(), ow_motion::InvalidSpring>(())
//! ```

mod animatable;
mod animated;
mod spring;

pub use animatable::Animatable;
pub use animated::Animated;
pub use spring::{InvalidSpring, Spring, Transition};
