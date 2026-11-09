//! Sizes: the few numbers every part of the interface measures with.
//!
//! All of them are in the interface's own unit, which is a pixel on a
//! screen of ordinary density and scales with the screen. Using these and
//! no others is what makes two panels written a year apart line up.

/// Distances between things and around them.
pub mod space {
    /// Between an icon and its label.
    pub const TINY: f64 = 2.0;
    /// Inside a small control.
    pub const SMALL: f64 = 4.0;
    /// Between controls that belong together.
    pub const COMPACT: f64 = 8.0;
    /// Around the content of a control, and between rows.
    pub const REGULAR: f64 = 12.0;
    /// Around the content of a panel.
    pub const ROOMY: f64 = 16.0;
    /// Between groups.
    pub const WIDE: f64 = 24.0;
    /// Between sections.
    pub const SECTION: f64 = 32.0;
    /// Around a page of content.
    pub const PAGE: f64 = 48.0;
}

/// How round corners are.
pub mod radius {
    /// Check boxes, tags, the cells of a grid.
    pub const SMALL: f64 = 4.0;
    /// Buttons and fields.
    pub const CONTROL: f64 = 6.0;
    /// Cards, menus and dialogs.
    pub const PANEL: f64 = 10.0;
    /// As round as the shape allows: a pill, a circle. A corner radius is
    /// cut down to what fits, so any size above the shape's does this.
    pub const ROUND: f64 = 10_000.0;
}

/// How tall controls are, and how thick lines.
pub mod control {
    /// A control in a dense place: a toolbar, a table.
    pub const SMALL: f64 = 24.0;
    /// A control.
    pub const REGULAR: f64 = 32.0;
    /// A control that is the main thing on its surface.
    pub const LARGE: f64 = 40.0;
    /// An edge or a separator.
    pub const LINE: f64 = 1.0;
    /// The ring around what has the keyboard, and its distance from it.
    pub const FOCUS: f64 = 2.0;
}

/// The size of text, the height of its lines and how heavy it is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Type {
    /// The size of the text.
    pub size: f64,
    /// The distance from one line to the next.
    pub line: f64,
    /// The weight of the font, 400 for regular and 600 for semibold, as
    /// font files and CSS count it.
    pub weight: u16,
}

impl Type {
    /// The small print under a field or a picture.
    pub const CAPTION: Self = Self::new(11.0, 16.0, 400);
    /// What most of the interface is written in.
    pub const BODY: Self = Self::new(13.0, 20.0, 400);
    /// A label that names what is beside it, and the text of a button.
    pub const LABEL: Self = Self::new(13.0, 20.0, 600);
    /// The title of a panel or a dialog.
    pub const TITLE: Self = Self::new(16.0, 24.0, 600);
    /// The heading of a page.
    pub const HEADING: Self = Self::new(22.0, 28.0, 600);
    /// The one large line of an empty or a welcome page.
    pub const DISPLAY: Self = Self::new(32.0, 40.0, 700);

    /// A style of text from its size, the height of its lines and its
    /// weight.
    pub const fn new(size: f64, line: f64, weight: u16) -> Self {
        Self { size, line, weight }
    }
}
