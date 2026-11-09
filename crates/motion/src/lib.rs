//! Motion for `OfficeWorld`.
//!
//! Everything that moves on screen gets its position from this crate. It
//! draws nothing and keeps no clock: it answers where a value is at a time
//! the caller names, in seconds.
//!
//! [`Spring`] is a damped spring solved in closed form. Its answer depends
//! only on the time asked for, not on how many frames were drawn before, so
//! an animation looks the same at any refresh rate.
//!
//! ```
//! use ow_motion::Spring;
//!
//! let spring = Spring::from_response(0.3, 1.0)?;
//! // 100 units away from the target and standing still, 0.2 seconds ago.
//! let (distance, velocity) = spring.transition(0.2).apply(100.0, 0.0);
//! assert!(distance > 0.0 && distance < 100.0);
//! assert!(velocity < 0.0);
//! # Ok::<(), ow_motion::InvalidSpring>(())
//! ```

mod animatable;
mod spring;

pub use animatable::Animatable;
pub use spring::{InvalidSpring, Spring, Transition};
