//! What a frame of `OfficeWorld` looks like, as data.
//!
//! The interface does not draw. It says what there is to see: this shape
//! in that colour, this group of things moved over there and cut off at
//! that edge. A [`Scene`] is that description. A renderer draws it, a PDF
//! writer can write it, and a test can read it.
//!
//! A scene is a list of items, drawn in order, and an item can be another
//! scene. That is what lets most of a window be kept from one frame to the
//! next: a list that scrolls is the same scene as before, placed somewhere
//! else.
//!
//! ```
//! use ow_color::Color;
//! use ow_geometry::{Affine, Rect, Vector};
//! use ow_path::{FillRule, Path, Stroke};
//! use ow_raster::Pixmap;
//! use ow_scene::{Group, Scene};
//!
//! // A button: a filled shape with a line around it.
//! let shape = Path::rect(Rect::new(0.0, 0.0, 30.0, 12.0));
//! let mut button = Scene::new();
//! button.fill(shape.clone(), FillRule::NonZero, Color::WHITE);
//! button.stroke(shape, Stroke::new(1.0), Color::BLACK);
//!
//! // A window with the button in it, 8 to the right and 20 down.
//! let mut window = Scene::new();
//! let place = Affine::translate(Vector::new(8.0, 20.0));
//! window.place(Group::new().with_transform(place), button);
//!
//! // What the window can reach: the button, and around it the room the
//! // pen's sharpest corner could take, which is 2 for this pen.
//! assert_eq!(window.bounds(), Some(Rect::new(6.0, 18.0, 40.0, 34.0)));
//!
//! // Drawn on a screen that shows two pixels to the unit.
//! let mut picture = Pixmap::new(96, 72)?;
//! window.draw(&mut picture, Affine::scale(2.0, 2.0));
//! assert_eq!(picture.pixel(40, 50), Some([255, 255, 255, 255]));
//! # Ok::<(), ow_raster::TooLarge>(())
//! ```
//!
//! # What is not built
//!
//! - A brush is one colour. Gradients and pictures as a fill are not here
//!   yet, and neither are pictures and runs of text as items.
//! - [`Scene::draw`] does not yet cut a group off at its clip and does not
//!   yet fade it: the reference renderer cannot do either so far. Both are
//!   kept in the scene and counted in its bounds.

mod scene;

pub use scene::{Brush, Group, Item, Scene};
