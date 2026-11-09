//! The platform layer of `OfficeWorld`.
//!
//! Everything a program needs from the system it runs on to show itself and
//! be used: windows, the pointer and the keyboard, the moment to draw the
//! next frame, and a way to put a picture on the screen. It is written here
//! for each system, and a program sees one interface whichever it runs on:
//! it implements [`App`], is handed every [`Event`] as it happens, and asks
//! for what it wants through [`Platform`].
//!
//! A program does not loop and does not read a clock. It is told when to
//! draw and for what time. That is what lets the same program be driven by
//! a screen at any refresh rate, and by [`Headless`] in a test: a platform
//! with no screen at all, where a test sends the events and looks at the
//! pictures.
//!
//! ```
//! use ow_platform::{App, Event, Headless, Picture, Platform, WindowEvent, WindowOptions};
//!
//! /// Opens a window and paints it white whenever it is given a size.
//! struct Blank;
//!
//! impl App for Blank {
//!     fn event(&mut self, platform: &mut dyn Platform, event: Event) {
//!         match event {
//!             Event::Started => {
//!                 let options = WindowOptions::new("Blank", 320.0, 200.0);
//!                 platform.open_window(options).expect("a window");
//!             }
//!             Event::Window(window, WindowEvent::Resized { width, height, .. }) => {
//!                 let white = vec![255; width as usize * height as usize * 4];
//!                 let picture = Picture::new(width, height, &white).expect("the right size");
//!                 platform.present(window, picture);
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//!
//! let mut screen = Headless::new(60.0);
//! screen.run(&mut Blank, 10);
//! let shown = screen.shown(screen.windows()[0]).expect("a picture");
//! assert_eq!((shown.width, shown.height, shown.pixels[0]), (320, 200, 255));
//! ```
//!
//! Built so far: this interface and [`Headless`]. The layers for X11,
//! Wayland, Windows and macOS are not built, and [`run`] says so.

mod event;
mod headless;

#[cfg(target_os = "macos")]
mod appkit;
#[cfg(all(unix, not(target_os = "macos")))]
mod unix;
#[cfg(all(unix, not(target_os = "macos")))]
mod wayland;
#[cfg(windows)]
mod win32;
#[cfg(all(unix, not(target_os = "macos")))]
mod x11;

use std::error::Error;
use std::fmt;
use std::sync::Arc;

pub use event::{
    Appearance, Button, Cursor, Event, Key, KeyInput, Modifiers, Pointer, PointerChange, Scroll,
    WindowEvent, WindowId,
};
pub use headless::{Headless, Shown};

/// A program, as a platform sees it: something that is told what happens.
pub trait App {
    /// Something happened. `platform` is there to answer it: to open a
    /// window, to ask for a frame, to present a picture.
    fn event(&mut self, platform: &mut dyn Platform, event: Event);
}

/// What a program can ask of the system it runs on.
///
/// A call that names a window which is not open does nothing.
pub trait Platform {
    /// Opens a window. Its first [`WindowEvent::Resized`] says how large it
    /// came out in pixels.
    ///
    /// # Errors
    ///
    /// Fails when the system refuses the window.
    fn open_window(&mut self, options: WindowOptions) -> Result<WindowId, PlatformError>;

    /// Closes a window. No more events come for it.
    fn close_window(&mut self, window: WindowId);

    /// Changes the title of a window.
    fn set_title(&mut self, window: WindowId, title: &str);

    /// Changes the shape of the pointer over a window.
    fn set_cursor(&mut self, window: WindowId, cursor: Cursor);

    /// Asks for one [`WindowEvent::Frame`] for a window, in step with the
    /// screen. Something that moves asks again from within each frame, and
    /// stops asking when it has come to rest: a window in which nothing
    /// moves is then not drawn at all.
    fn request_frame(&mut self, window: WindowId);

    /// Puts a picture into a window. A picture of another size than the
    /// window is placed at the top left corner and not stretched.
    fn present(&mut self, window: WindowId, picture: Picture<'_>);

    /// The text on the clipboard, if it holds text.
    fn clipboard_text(&mut self) -> Option<String>;

    /// Puts text on the clipboard.
    fn set_clipboard_text(&mut self, text: &str);

    /// How the system wants programs to look right now.
    fn appearance(&self) -> Appearance;

    /// The time now, in seconds on the clock that
    /// [`WindowEvent::Frame`] counts on. Only differences between two
    /// times mean anything.
    fn now(&self) -> f64;

    /// Asks for an [`Event::Woken`] at a time on that clock, for something
    /// that is due later without a frame: a cursor that blinks, a hint
    /// that appears after a pause.
    fn wake_at(&mut self, time: f64);

    /// A handle another thread can wake the program with: work that was
    /// done in the background sends its result and then wakes the program
    /// to look at it.
    fn waker(&self) -> Waker;

    /// Ends the program: [`run`] returns once the event being handled is
    /// done.
    fn quit(&mut self);
}

/// What a window is opened with.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowOptions {
    /// The title.
    pub title: String,
    /// The width of the part the program draws, in units of the interface.
    /// The system may give the window another size.
    pub width: f64,
    /// The height of the part the program draws, in units of the interface.
    pub height: f64,
    /// Whether the person can change the size of the window.
    pub resizable: bool,
}

impl WindowOptions {
    /// A window with a title and a size in units of the interface, whose
    /// size the person can change.
    pub fn new(title: &str, width: f64, height: f64) -> Self {
        Self {
            title: title.to_owned(),
            width,
            height,
            resizable: true,
        }
    }
}

/// A picture to put into a window: rows of pixels from the top, each a
/// red, a green, a blue and an alpha byte, the colour multiplied by its
/// alpha. It is the form the renderer's pictures are stored in. A window
/// has nothing behind it, so what a picture leaves transparent shows black.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Picture<'a> {
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
    pub height: u32,
    /// The bytes: four for each pixel.
    pub pixels: &'a [u8],
}

impl<'a> Picture<'a> {
    /// A picture of a size from its bytes, or `None` if there are not
    /// exactly four bytes for each pixel.
    pub fn new(width: u32, height: u32, pixels: &'a [u8]) -> Option<Self> {
        let needed = u64::from(width) * u64::from(height) * 4;
        let fits = u64::try_from(pixels.len()).is_ok_and(|length| length == needed);
        fits.then_some(Self {
            width,
            height,
            pixels,
        })
    }
}

/// A handle that wakes a program from another thread.
#[derive(Clone)]
pub struct Waker(Arc<dyn Fn() + Send + Sync>);

impl Waker {
    /// A waker that calls a function. A platform makes these; a program
    /// gets one from [`Platform::waker`].
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
        Self(Arc::new(wake))
    }

    /// Wakes the program: it is sent an [`Event::Woken`] soon.
    pub fn wake(&self) {
        (self.0)();
    }
}

impl fmt::Debug for Waker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Waker")
    }
}

/// Why the platform could not do what was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    /// There is no display to show anything on, or it cannot be reached.
    /// The text says what was tried.
    NoDisplay(String),
    /// The system refused. The text says what, in the system's words where
    /// it has any.
    Refused(String),
    /// The connection to the display was lost while the program ran.
    Lost,
    /// The layer for this system, named here, is not built yet.
    NotBuilt(&'static str),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDisplay(tried) => write!(f, "There is no display to open a window on: {tried}"),
            Self::Refused(what) => write!(f, "The system refused: {what}"),
            Self::Lost => f.write_str("The connection to the display was lost."),
            Self::NotBuilt(system) => write!(f, "Windows on {system} are not built yet."),
        }
    }
}

impl Error for PlatformError {}

/// Runs a program on the system's display: sends it [`Event::Started`] and
/// then every event as it happens, until it calls [`Platform::quit`].
///
/// # Errors
///
/// Fails when there is no display to connect to, when the connection is
/// lost, and on a system whose layer is not built yet.
pub fn run(app: &mut dyn App) -> Result<(), PlatformError> {
    #[cfg(windows)]
    let done = win32::run(app);
    #[cfg(target_os = "macos")]
    let done = appkit::run(app);
    #[cfg(all(unix, not(target_os = "macos")))]
    let done = unix::run(app);
    done
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_error_says_what_went_wrong() {
        let errors = [
            PlatformError::NoDisplay("DISPLAY is not set.".to_owned()),
            PlatformError::Refused("no more windows".to_owned()),
            PlatformError::Lost,
            PlatformError::NotBuilt("X11"),
        ];
        let said = errors.map(|error| error.to_string());
        let expected = [
            "There is no display to open a window on: DISPLAY is not set.",
            "The system refused: no more windows",
            "The connection to the display was lost.",
            "Windows on X11 are not built yet.",
        ];
        assert_eq!(said, expected);
        assert_eq!(format!("{:?}", Waker::new(|| {})), "Waker");
    }
}
