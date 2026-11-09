//! The layer for macOS. Not built yet.

use crate::{App, PlatformError};

/// Runs a program on macOS.
pub(crate) fn run(_app: &mut dyn App) -> Result<(), PlatformError> {
    Err(PlatformError::NotBuilt("macOS"))
}
