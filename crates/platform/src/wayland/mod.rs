//! The layer for Wayland. Not built yet.

use crate::{App, PlatformError};

/// Runs a program on Wayland.
pub(crate) fn run(_app: &mut dyn App) -> Result<(), PlatformError> {
    Err(PlatformError::NotBuilt("Wayland"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, Platform};

    struct Idle;

    impl App for Idle {
        fn event(&mut self, _platform: &mut dyn Platform, _event: Event) {}
    }

    #[test]
    fn the_layer_says_that_it_is_not_built() {
        assert_eq!(run(&mut Idle), Err(PlatformError::NotBuilt("Wayland")));
    }
}
