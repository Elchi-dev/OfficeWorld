//! What a system tells a program: the events.

/// One of the program's windows. The number means nothing outside the
/// platform that gave it out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(pub u32);

/// Something that happened.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The program is running and may open its windows. It comes once,
    /// before anything else.
    Started,
    /// Something happened to a window.
    Window(WindowId, WindowEvent),
    /// How the system wants programs to look has changed.
    Appearance(Appearance),
    /// The program asked to be woken, with [`Waker::wake`](crate::Waker) or
    /// with [`Platform::wake_at`](crate::Platform::wake_at).
    Woken,
}

/// Something that happened to a window.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// The window has a size, for the first time or a new one. `width` and
    /// `height` are in pixels of the screen, and one unit of the interface
    /// is `scale` of them. A picture presented from now on has this size.
    Resized {
        /// The width of the part the program draws, in pixels.
        width: u32,
        /// The height of the part the program draws, in pixels.
        height: u32,
        /// How many pixels one unit of the interface is: 1 on an ordinary
        /// screen, 2 on one of double density, and anything between.
        scale: f64,
    },
    /// It is time to draw. It comes once for every
    /// [`Platform::request_frame`](crate::Platform::request_frame), in step
    /// with the screen. `time` is when the picture drawn now will be seen,
    /// in seconds on the platform's clock, and is the time everything that
    /// moves should be drawn for.
    Frame {
        /// When the picture drawn now will be seen.
        time: f64,
    },
    /// A part of the window has to be drawn again because the system lost
    /// it. Present the picture once more.
    Damaged,
    /// The person asked to close the window. It stays open until the
    /// program closes it.
    CloseRequested,
    /// The window now takes the keyboard, or no longer does.
    Focused(bool),
    /// The pointer moved, entered, left, or one of its buttons changed.
    Pointer(Pointer),
    /// A wheel was turned or two fingers moved on a touchpad.
    Scroll(Scroll),
    /// A key went down or up.
    Key(KeyInput),
}

/// The pointer at one moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pointer {
    /// How far the pointer is from the left edge of the window, in pixels.
    pub x: f64,
    /// How far the pointer is from the top edge of the window, in pixels.
    pub y: f64,
    /// What changed.
    pub change: PointerChange,
    /// The modifier keys held at that moment.
    pub modifiers: Modifiers,
}

/// What a pointer event reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerChange {
    /// The pointer came over the window.
    Entered,
    /// The pointer left the window.
    Left,
    /// The pointer moved.
    Moved,
    /// A button went down.
    Pressed(Button),
    /// A button came up.
    Released(Button),
}

/// A button of a mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// The button that selects: the left one for most people.
    Primary,
    /// The button that opens a menu: the right one for most people.
    Secondary,
    /// The wheel pressed down, or the button in the middle.
    Middle,
    /// The side button that goes back.
    Back,
    /// The side button that goes forward.
    Forward,
}

/// A turn of a wheel or a move of two fingers on a touchpad.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scroll {
    /// Where the pointer is, in pixels from the left edge of the window.
    pub x: f64,
    /// Where the pointer is, in pixels from the top edge of the window.
    pub y: f64,
    /// How far to scroll to the right, in pixels. Negative is to the left.
    pub right: f64,
    /// How far to scroll down, in pixels: towards what lies further down
    /// the page. Negative is up.
    pub down: f64,
    /// Whether the distances follow fingers on a touchpad exactly. If not,
    /// they are the steps of a wheel, already turned into pixels.
    pub exact: bool,
    /// The modifier keys held at that moment.
    pub modifiers: Modifiers,
}

/// A key going down or up.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyInput {
    /// Which key, by what it means on this person's keyboard layout.
    pub key: Key,
    /// The key by where it is on the keyboard, as the system numbers it.
    /// The numbers differ between systems.
    pub scancode: u32,
    /// Whether the key went down, or came up.
    pub pressed: bool,
    /// Whether this is the key repeating because it is being held.
    pub repeat: bool,
    /// The modifier keys held at that moment.
    pub modifiers: Modifiers,
    /// The text the key types, if it types any and went down: what a text
    /// field should insert. Keys held together with Control or the logo
    /// key type nothing.
    pub text: Option<String>,
}

/// A key, by what it means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A key that stands for a character: the one it gives with the
    /// modifiers that choose among its characters (Shift, `AltGr`) and
    /// without the ones that make it a command (Control, Alt, the logo
    /// key). The space bar is `Character(' ')`.
    Character(char),
    /// Enter or Return.
    Enter,
    /// The tabulator.
    Tab,
    /// Backspace: delete what is before the cursor.
    Backspace,
    /// Delete: delete what is after the cursor.
    Delete,
    /// Escape.
    Escape,
    /// Insert.
    Insert,
    /// The arrow to the left.
    Left,
    /// The arrow to the right.
    Right,
    /// The arrow up.
    Up,
    /// The arrow down.
    Down,
    /// Home: to the start of the line.
    Home,
    /// End: to the end of the line.
    End,
    /// A page up.
    PageUp,
    /// A page down.
    PageDown,
    /// A function key, `Function(1)` for F1.
    Function(u8),
    /// Either Shift key.
    Shift,
    /// Either Control key.
    Control,
    /// Either Alt key, Option on a Mac.
    Alt,
    /// The key with the system's logo: Windows, Command or Super.
    Logo,
    /// Caps Lock.
    CapsLock,
    /// The key that opens a context menu.
    Menu,
    /// A key this layer has no name for yet.
    Unknown,
}

/// The modifier keys that are held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "four keys, each held or not, and none of them depends on another"
)]
pub struct Modifiers {
    /// Shift.
    pub shift: bool,
    /// Control.
    pub control: bool,
    /// Alt, Option on a Mac.
    pub alt: bool,
    /// The key with the system's logo: Windows, Command or Super.
    pub logo: bool,
}

/// How the system wants programs to look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Appearance {
    /// Whether the person chose a dark look.
    pub dark: bool,
    /// Whether the person asked for less motion.
    pub reduced_motion: bool,
}

/// The shape of the pointer over a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Cursor {
    /// The arrow.
    #[default]
    Arrow,
    /// The beam over text.
    Text,
    /// The hand over something that can be followed.
    Hand,
    /// An open hand over something that can be moved.
    Grab,
    /// A closed hand while it is being moved.
    Grabbing,
    /// Crossed lines for placing something exactly.
    Crosshair,
    /// Arrows left and right, over an edge that moves sideways.
    ResizeHorizontal,
    /// Arrows up and down, over an edge that moves up and down.
    ResizeVertical,
    /// The sign that something is not allowed here.
    NotAllowed,
    /// No pointer at all, while typing or while something is dragged that
    /// shows where it is by itself.
    Hidden,
}
