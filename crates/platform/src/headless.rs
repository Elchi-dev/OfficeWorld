//! A platform without a screen, for tests.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::event::{Appearance, Cursor, Event, WindowEvent, WindowId};
use crate::{App, Picture, Platform, PlatformError, Waker, WindowOptions};

/// A picture a program put into a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
    pub height: u32,
    /// The bytes as they were presented: four for each pixel.
    pub pixels: Vec<u8>,
}

struct Window {
    id: WindowId,
    title: String,
    cursor: Cursor,
    wants_frame: bool,
    shown: Option<Shown>,
    presented: u32,
}

/// A platform with no screen behind it: a test sends the events a system
/// would send and looks at what the program does with them.
///
/// Its clock only moves when a frame is drawn or a wake-up is due, by
/// exactly as much as that takes, so a test of something that moves gives
/// the same result every time it runs.
pub struct Headless {
    /// The seconds from one frame to the next.
    interval: f64,
    scale: f64,
    clock: f64,
    windows: Vec<Window>,
    opened: u32,
    waiting: VecDeque<Event>,
    started: bool,
    quit: bool,
    clipboard: Option<String>,
    appearance: Appearance,
    wake_times: Vec<f64>,
    woken: Arc<AtomicBool>,
}

impl Headless {
    /// A platform whose screen shows so many frames a second and has one
    /// pixel to a unit of the interface.
    pub fn new(frames_per_second: f64) -> Self {
        Self {
            interval: 1.0 / frames_per_second,
            scale: 1.0,
            clock: 0.0,
            windows: Vec::new(),
            opened: 0,
            waiting: VecDeque::new(),
            started: false,
            quit: false,
            clipboard: None,
            appearance: Appearance::default(),
            wake_times: Vec::new(),
            woken: Arc::new(AtomicBool::new(false)),
        }
    }

    /// The same platform with a screen of another density: so many pixels
    /// to a unit of the interface.
    #[must_use]
    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    /// Sends the program an event as the system would. It is delivered by
    /// the next [`run`](Self::run). A change of appearance sent this way is
    /// also what [`Platform::appearance`] answers from then on.
    pub fn send(&mut self, event: Event) {
        if let Event::Appearance(appearance) = event {
            self.appearance = appearance;
        }
        self.waiting.push_back(event);
    }

    /// Lets the program run until it has nothing left to do: delivers the
    /// events that wait, then draws the frames the program asks for and
    /// wakes it when it asked to be woken, moving the clock forward as it
    /// goes. It stops when the program asks for nothing more, when it
    /// quits, or after `turns` frames and wake-ups, so a program that never
    /// comes to rest cannot hold a test for ever. Returns how many frames
    /// were drawn.
    pub fn run(&mut self, app: &mut dyn App, turns: u32) -> u32 {
        if !self.started {
            self.started = true;
            self.waiting.push_front(Event::Started);
        }
        let mut frames = 0;
        for _ in 0..turns {
            self.deliver(app);
            frames += u32::from(self.next_turn());
        }
        self.deliver(app);
        frames
    }

    /// Hands the program every event that waits, and those that come up
    /// while it answers them.
    fn deliver(&mut self, app: &mut dyn App) {
        while let Some(event) = self.next() {
            app.event(self, event);
        }
    }

    /// The next event to deliver: none after the program quit, and none
    /// for a window that is closed.
    fn next(&mut self) -> Option<Event> {
        if self.woken.swap(false, Ordering::Relaxed) {
            self.waiting.push_back(Event::Woken);
        }
        let open = |windows: &[Window], event: &Event| match event {
            Event::Window(id, _) => windows.iter().any(|window| window.id == *id),
            _ => true,
        };
        while let Some(event) = self.waiting.pop_front() {
            if !self.quit && open(&self.windows, &event) {
                return Some(event);
            }
        }
        None
    }

    /// Queues what is due next: a frame for the windows that asked for one,
    /// or else the earliest wake-up. Says whether it was a frame.
    fn next_turn(&mut self) -> bool {
        let asking = self.windows.iter_mut().filter(|window| window.wants_frame);
        let asking: Vec<WindowId> = asking.map(|window| window.id).collect();
        if asking.is_empty() {
            let earliest = self
                .wake_times
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);
            if earliest.is_finite() {
                self.wake_times.retain(|time| *time > earliest);
                self.clock = self.clock.max(earliest);
                self.waiting.push_back(Event::Woken);
            }
            return false;
        }
        self.clock += self.interval;
        for id in asking {
            let frame = WindowEvent::Frame { time: self.clock };
            self.waiting.push_back(Event::Window(id, frame));
        }
        for window in &mut self.windows {
            window.wants_frame = false;
        }
        true
    }

    fn window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|window| window.id == id)
    }

    fn window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|window| window.id == id)
    }

    /// The windows that are open, in the order they were opened.
    pub fn windows(&self) -> Vec<WindowId> {
        self.windows.iter().map(|window| window.id).collect()
    }

    /// The picture a window shows: the last one presented to it.
    pub fn shown(&self, window: WindowId) -> Option<&Shown> {
        self.window(window)?.shown.as_ref()
    }

    /// How many pictures have been presented to a window.
    pub fn presented(&self, window: WindowId) -> u32 {
        self.window(window).map_or(0, |window| window.presented)
    }

    /// The title of a window.
    pub fn title(&self, window: WindowId) -> Option<&str> {
        Some(self.window(window)?.title.as_str())
    }

    /// The shape of the pointer over a window.
    pub fn cursor(&self, window: WindowId) -> Option<Cursor> {
        Some(self.window(window)?.cursor)
    }

    /// Whether the program has quit.
    pub fn has_quit(&self) -> bool {
        self.quit
    }
}

impl Platform for Headless {
    fn open_window(&mut self, options: WindowOptions) -> Result<WindowId, PlatformError> {
        let id = WindowId(self.opened);
        self.opened += 1;
        self.windows.push(Window {
            id,
            title: options.title,
            cursor: Cursor::default(),
            wants_frame: false,
            shown: None,
            presented: 0,
        });
        // A real system answers a new window with its size in pixels.
        let pixels = |units: f64| {
            let rounded = (units * self.scale).round().clamp(1.0, 65_535.0);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "the value was just brought between 1 and 65,535"
            )]
            let whole = rounded as u32;
            whole
        };
        let resized = WindowEvent::Resized {
            width: pixels(options.width),
            height: pixels(options.height),
            scale: self.scale,
        };
        self.waiting.push_back(Event::Window(id, resized));
        Ok(id)
    }

    fn close_window(&mut self, window: WindowId) {
        self.windows.retain(|open| open.id != window);
    }

    fn set_title(&mut self, window: WindowId, title: &str) {
        if let Some(window) = self.window_mut(window) {
            title.clone_into(&mut window.title);
        }
    }

    fn set_cursor(&mut self, window: WindowId, cursor: Cursor) {
        if let Some(window) = self.window_mut(window) {
            window.cursor = cursor;
        }
    }

    fn request_frame(&mut self, window: WindowId) {
        if let Some(window) = self.window_mut(window) {
            window.wants_frame = true;
        }
    }

    fn present(&mut self, window: WindowId, picture: Picture<'_>) {
        if let Some(window) = self.window_mut(window) {
            window.presented += 1;
            window.shown = Some(Shown {
                width: picture.width,
                height: picture.height,
                pixels: picture.pixels.to_vec(),
            });
        }
    }

    fn clipboard_text(&mut self) -> Option<String> {
        self.clipboard.clone()
    }

    fn set_clipboard_text(&mut self, text: &str) {
        self.clipboard = Some(text.to_owned());
    }

    fn appearance(&self) -> Appearance {
        self.appearance
    }

    fn now(&self) -> f64 {
        self.clock
    }

    fn wake_at(&mut self, time: f64) {
        self.wake_times.push(time);
    }

    fn waker(&self) -> Waker {
        let woken = Arc::clone(&self.woken);
        Waker::new(move || woken.store(true, Ordering::Relaxed))
    }

    fn quit(&mut self) {
        self.quit = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A program that writes down what it is told and answers each event
    /// with a function.
    struct Script<F: FnMut(&mut dyn Platform, &Event)> {
        told: Vec<Event>,
        answer: F,
    }

    impl<F: FnMut(&mut dyn Platform, &Event)> App for Script<F> {
        fn event(&mut self, platform: &mut dyn Platform, event: Event) {
            (self.answer)(platform, &event);
            self.told.push(event);
        }
    }

    fn script<F: FnMut(&mut dyn Platform, &Event)>(answer: F) -> Script<F> {
        let told = Vec::new();
        Script { told, answer }
    }

    const WINDOW: WindowId = WindowId(0);

    #[test]
    fn frames_come_as_long_as_they_are_asked_for_and_no_longer() {
        // A program that moves for three frames: it asks for the next
        // frame from within each one, and then it has come to rest.
        let mut left = 3;
        let mut app = script(|platform, event| match event {
            Event::Started => {
                let options = WindowOptions::new("Moving", 100.4, 50.0);
                platform.open_window(options).unwrap();
                platform.request_frame(WINDOW);
            }
            Event::Window(window, WindowEvent::Frame { .. }) if left > 1 => {
                left -= 1;
                platform.request_frame(*window);
            }
            _ => {}
        });
        let mut screen = Headless::new(50.0).with_scale(1.5);
        assert_eq!(
            (screen.run(&mut app, 100), screen.run(&mut app, 100)),
            (3, 0)
        );
        // It was told its size in pixels first, then each frame with the
        // time it would be seen at, a fiftieth of a second apart.
        let resized = WindowEvent::Resized {
            width: 151,
            height: 75,
            scale: 1.5,
        };
        let frame = |time| Event::Window(WINDOW, WindowEvent::Frame { time });
        let expected = [
            Event::Started,
            Event::Window(WINDOW, resized),
            frame(0.02),
            frame(0.04),
            frame(0.06),
        ];
        assert_eq!((app.told.as_slice(), screen.now()), (&expected[..], 0.06));
        // A program that never comes to rest is stopped after the turns it
        // was given, and goes on from there the next time.
        let mut restless = script(|platform, event| match event {
            Event::Started => drop(platform.open_window(WindowOptions::new("", 1.0, 1.0))),
            _ => platform.request_frame(WINDOW),
        });
        let mut screen = Headless::new(60.0);
        assert_eq!(
            (screen.run(&mut restless, 7), screen.run(&mut restless, 2)),
            (7, 2)
        );
    }

    #[test]
    fn a_program_is_woken_when_it_asked_and_from_other_threads() {
        let mut app = script(|platform, event| {
            if *event == Event::Started {
                platform.wake_at(2.5);
                platform.wake_at(1.0);
                platform.wake_at(1.0);
            }
        });
        let mut screen = Headless::new(60.0);
        // The clock jumps to each wake-up in turn. Two for the same time
        // are one.
        assert_eq!((screen.run(&mut app, 1), screen.now()), (0, 1.0));
        assert_eq!((screen.run(&mut app, 5), screen.now()), (0, 2.5));
        assert_eq!(app.told, [Event::Started, Event::Woken, Event::Woken]);
        let waker = screen.waker();
        std::thread::spawn(move || waker.wake()).join().unwrap();
        screen.run(&mut app, 5);
        assert_eq!((app.told.len(), screen.now()), (4, 2.5));
    }

    #[test]
    fn a_window_keeps_what_it_was_given_until_it_is_closed() {
        let mut app = script(|platform, event| match event {
            Event::Started => {
                platform
                    .open_window(WindowOptions::new("One", 2.0, 1.0))
                    .unwrap();
                platform
                    .open_window(WindowOptions::new("Two", 2.0, 1.0))
                    .unwrap();
            }
            Event::Window(window, WindowEvent::Resized { width, height, .. }) => {
                let grey = [9; 8];
                platform.present(*window, Picture::new(*width, *height, &grey).unwrap());
                platform.set_title(*window, "Renamed");
                platform.set_cursor(*window, Cursor::Text);
                platform.set_clipboard_text("copied");
            }
            Event::Window(window, WindowEvent::CloseRequested) => platform.close_window(*window),
            Event::Appearance(_) => platform.quit(),
            _ => {}
        });
        let mut screen = Headless::new(60.0);
        screen.run(&mut app, 5);
        let second = WindowId(1);
        assert_eq!(screen.windows(), [WINDOW, second]);
        let shown = screen.shown(second).unwrap();
        assert_eq!(
            (shown.width, shown.height, &shown.pixels[..]),
            (2, 1, &[9; 8][..])
        );
        let kept = (
            screen.title(second),
            screen.cursor(second),
            screen.presented(second),
        );
        assert_eq!(kept, (Some("Renamed"), Some(Cursor::Text), 1));
        assert_eq!(screen.clipboard_text().as_deref(), Some("copied"));
        // A closed window is gone, and what was still on its way to it is
        // not delivered.
        screen.send(Event::Window(second, WindowEvent::CloseRequested));
        screen.send(Event::Window(second, WindowEvent::Focused(true)));
        screen.run(&mut app, 5);
        assert_eq!((screen.windows(), app.told.len()), (vec![WINDOW], 4));
        let gone = (
            screen.shown(second),
            screen.title(second),
            screen.cursor(second),
        );
        assert_eq!((gone, screen.presented(second)), ((None, None, None), 0));
        // After the program quits it is told nothing more.
        let dark = Appearance {
            dark: true,
            reduced_motion: false,
        };
        assert!(!screen.has_quit() && !screen.appearance().dark);
        screen.send(Event::Appearance(dark));
        screen.send(Event::Woken);
        screen.run(&mut app, 5);
        assert!(screen.has_quit() && screen.appearance().dark && app.told.len() == 5);
        // A picture has to have four bytes for each of its pixels.
        assert!(Picture::new(2, 1, &[0; 7]).is_none() && Picture::new(0, 9, &[]).is_some());
    }
}
