//! Drawing for `OfficeWorld` on the CPU.
//!
//! This is the reference renderer: it fills paths into pictures in memory,
//! exactly and the same on every machine. It is what exports, thumbnails
//! and tests draw with, and what the renderer on the GPU is measured
//! against.

mod mask;

pub use mask::Mask;
