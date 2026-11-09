//! The two ways a Unix desktop shows windows, and the choice between them.

use crate::{App, PlatformError, wayland, x11};

/// Runs a program on the display of a Unix desktop.
pub(crate) fn run(app: &mut dyn App) -> Result<(), PlatformError> {
    // A Wayland session says so in the environment.
    run_on(std::env::var_os("WAYLAND_DISPLAY").is_some(), app)
}

/// Runs a program on Wayland in a Wayland session and on X11 otherwise.
/// Until the layer for Wayland is built, X11 is spoken there as well: a
/// Wayland session understands it.
fn run_on(wayland_session: bool, app: &mut dyn App) -> Result<(), PlatformError> {
    match wayland_session.then(|| wayland::run(app)) {
        None | Some(Err(PlatformError::NotBuilt(_))) => x11::run(app),
        Some(done) => done,
    }
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
    fn either_session_ends_at_the_layer_that_is_missing() {
        let x11 = Err(PlatformError::NotBuilt("X11"));
        let sessions = (run_on(true, &mut Idle), run_on(false, &mut Idle));
        assert_eq!(sessions, (x11.clone(), x11.clone()));
        assert_eq!(crate::run(&mut Idle), x11);
    }
}
