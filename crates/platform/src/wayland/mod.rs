//! The layer for Wayland. Not built yet.

use crate::{App, PlatformError};

/// Runs a program on Wayland.
pub(crate) fn run(_app: &mut dyn App) -> Result<(), PlatformError> {
    Err(PlatformError::NotBuilt("Wayland"))
}
