//! The layer for X11. Not built yet.

use crate::{App, PlatformError};

/// Runs a program on X11.
pub(crate) fn run(_app: &mut dyn App) -> Result<(), PlatformError> {
    Err(PlatformError::NotBuilt("X11"))
}
