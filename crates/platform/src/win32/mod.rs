//! The layer for Windows. Not built yet.

use crate::{App, PlatformError};

/// Runs a program on Windows.
pub(crate) fn run(_app: &mut dyn App) -> Result<(), PlatformError> {
    Err(PlatformError::NotBuilt("Windows"))
}
