//! Scenes: what there is to see, in the order it is drawn.

use std::f64::consts::SQRT_2;
use std::sync::Arc;

use ow_color::Color;
use ow_geometry::{Affine, Insets, Rect};
use ow_path::{Cap, FillRule, Join, Path, Stroke};
use ow_raster::Pixmap;

/// How far the straight lines a curve is drawn with may stray from it, in
/// pixels of the picture that is drawn into.
const TOLERANCE: f64 = 0.1;

/// What a shape is filled with.
///
/// A colour turns into a brush by itself, so it can be handed to
/// [`Scene::fill`] as it is.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Brush {
    /// The same colour everywhere.
    Color(Color),
}

impl From<Color> for Brush {
    fn from(color: Color) -> Self {
        Self::Color(color)
    }
}

/// How a scene is placed in another one.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// Where the scene goes: from its own coordinates to those of the
    /// scene it is placed in.
    pub transform: Affine,
    /// What is left of the scene: the part inside this path, which is in
    /// the scene's own coordinates and filled by the non-zero rule.
    /// Without one, all of it is left.
    pub clip: Option<Path>,
    /// How much of the scene shows, from 0 for nothing to 1 for all of
    /// it. The scene is faded as one picture, so shapes in it that overlap
    /// do not show through each other.
    pub opacity: f32,
}

impl Group {
    /// A scene where it is, all of it, not faded.
    pub const fn new() -> Self {
        Self {
            transform: Affine::IDENTITY,
            clip: None,
            opacity: 1.0,
        }
    }

    /// The same placing, with the scene moved, scaled or turned.
    #[must_use]
    pub fn with_transform(self, transform: Affine) -> Self {
        Self { transform, ..self }
    }

    /// The same placing, with only the part of the scene inside a path
    /// left.
    #[must_use]
    pub fn with_clip(self, clip: Path) -> Self {
        let clip = Some(clip);
        Self { clip, ..self }
    }

    /// The same placing, with the scene faded.
    #[must_use]
    pub fn with_opacity(self, opacity: f32) -> Self {
        Self { opacity, ..self }
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

/// One thing in a scene.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Item {
    /// A path, filled.
    Fill {
        /// The outline.
        path: Path,
        /// Which parts of an outline that crosses itself are inside.
        rule: FillRule,
        /// What it is filled with.
        brush: Brush,
    },
    /// A path, drawn along with a pen.
    Stroke {
        /// The line the pen follows.
        path: Path,
        /// The pen.
        pen: Stroke,
        /// What the pen leaves.
        brush: Brush,
    },
    /// Another scene, placed in this one.
    Group {
        /// How it is placed.
        group: Group,
        /// The scene. It is shared and not copied, so one that did not
        /// change can be placed again frame after frame.
        content: Arc<Scene>,
    },
}

/// What there is to see: items, drawn in order, later ones over earlier
/// ones.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scene {
    items: Vec<Item>,
    bounds: Option<Rect>,
}

impl Scene {
    /// A scene with nothing in it.
    pub const fn new() -> Self {
        Self {
            items: Vec::new(),
            bounds: None,
        }
    }

    /// Adds a filled path.
    pub fn fill(&mut self, path: Path, rule: FillRule, brush: impl Into<Brush>) {
        self.reach(path.bounds());
        let brush = brush.into();
        self.items.push(Item::Fill { path, rule, brush });
    }

    /// Adds a path drawn along with a pen. The pen's width is in the
    /// scene's units, like the path.
    pub fn stroke(&mut self, path: Path, pen: Stroke, brush: impl Into<Brush>) {
        // A mitered corner reaches furthest, as far as the pen's limit
        // lets it, and a square end reaches to its corners.
        let corner = match pen.join {
            Join::Miter => pen.miter_limit.max(1.0),
            Join::Round | Join::Bevel => 1.0,
        };
        let end = match pen.cap {
            Cap::Square => SQRT_2,
            Cap::Butt | Cap::Round => 1.0,
        };
        let reach = Insets::uniform(-(pen.width / 2.0).max(0.0) * corner.max(end));
        self.reach(path.bounds().map(|bounds| bounds.inset(reach)));
        let brush = brush.into();
        self.items.push(Item::Stroke { path, pen, brush });
    }

    /// Adds another scene, placed as a group says.
    pub fn place(&mut self, group: Group, content: impl Into<Arc<Self>>) {
        let content = content.into();
        let kept = match (&group.clip, content.bounds) {
            (Some(clip), Some(bounds)) => clip.bounds().map(|clip| clip.intersect(bounds)),
            (None, bounds) => bounds,
            (Some(_), None) => None,
        };
        let kept = kept.filter(|kept| !kept.is_empty());
        self.reach(kept.map(|kept| group.transform.bounds_of(kept)));
        self.items.push(Item::Group { group, content });
    }

    /// The items, in the order they are drawn.
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// A rectangle nothing in the scene reaches out of, or `None` for a
    /// scene that shows nothing. It can be larger than what is drawn: a
    /// stroke is given the room its sharpest corner could take.
    pub const fn bounds(&self) -> Option<Rect> {
        self.bounds
    }

    /// Draws the scene into a picture. The transform takes the scene's
    /// units to pixels: a scale of 2 for a screen that shows two pixels to
    /// the unit.
    ///
    /// Not built: a group is drawn whole where it has a clip, and in full
    /// where it is faded.
    pub fn draw(&self, picture: &mut Pixmap, transform: Affine) {
        for item in &self.items {
            match item {
                Item::Fill { path, rule, brush } => {
                    let Brush::Color(color) = *brush;
                    picture.fill(&path.transformed(transform), *rule, color);
                }
                Item::Stroke { path, pen, brush } => {
                    // The pen is as wide as the scene says in the scene's
                    // units. So the outline of the stroke is made there,
                    // as fine as the pixels it ends up on need it, and
                    // then moved like everything else.
                    let Brush::Color(color) = *brush;
                    let stretch = transform.determinant().abs().sqrt();
                    let outline = path.stroke(*pen, TOLERANCE / stretch);
                    let outline = outline.transformed(transform);
                    picture.fill(&outline, FillRule::NonZero, color);
                }
                Item::Group { group, content } => {
                    content.draw(picture, transform * group.transform);
                }
            }
        }
    }

    /// Lets the bounds take in what a new item reaches.
    fn reach(&mut self, bounds: Option<Rect>) {
        let bounds = bounds.filter(|bounds| !bounds.is_empty());
        let both = self.bounds.zip(bounds).map(|(old, new)| old.union(new));
        self.bounds = both.or(self.bounds).or(bounds);
    }
}

#[cfg(test)]
mod tests {
    use ow_geometry::Vector;

    use super::*;

    #[test]
    fn a_scene_draws_what_its_parts_draw_when_placed_by_hand() {
        let shape: Path = "M2 2 H20 V12 H12 V1 Z".parse().unwrap();
        let line: Path = "M3 20 Q12 4 21 20".parse().unwrap();
        let pen = Stroke::new(2.0).with_cap(Cap::Round);
        let blue = Color::from_hex("#1a73e880").unwrap();
        let grow = Affine::scale(1.5, 1.5);
        let shift = Affine::translate(Vector::new(3.0, 1.0));

        let mut inner = Scene::new();
        inner.stroke(line.clone(), pen, blue);
        let mut middle = Scene::new();
        middle.fill(shape.clone(), FillRule::EvenOdd, Color::BLACK);
        middle.place(Group::new().with_transform(grow), inner);
        let mut scene = Scene::new();
        scene.place(Group::new().with_transform(shift), middle);
        assert!(matches!(scene.items(), [Item::Group { .. }]));

        // Onto a screen with two pixels to the unit, and onto one that is
        // also mirrored. The pen is drawn in the scene's units, three
        // times finer than the pixels, which are three times smaller.
        let mirror = Affine::translate(Vector::new(80.0, 0.0)) * Affine::scale(-2.0, 2.0);
        for screen in [Affine::scale(2.0, 2.0), mirror] {
            let mut drawn = Pixmap::new(80, 72).unwrap();
            scene.draw(&mut drawn, screen);
            let mut expected = Pixmap::new(80, 72).unwrap();
            assert_ne!(drawn, expected);
            let placed = shape.transformed(screen * shift);
            expected.fill(&placed, FillRule::EvenOdd, Color::BLACK);
            let outline = line.stroke(pen, 0.1 / 3.0);
            let placed = outline.transformed(screen * shift * grow);
            expected.fill(&placed, FillRule::NonZero, blue);
            assert!(drawn == expected, "{screen:?}");
        }
    }

    #[test]
    fn the_bounds_leave_a_stroke_the_room_its_pen_can_take() {
        // A corner of 53 degrees and two open ends. The last pen has a
        // width below nothing and leaves nothing beside the path.
        let bend: Path = "M0 0 L10 5 L0 10".parse().unwrap();
        let pens = [
            (2.0, Join::Miter, Cap::Butt, 4.0, 4.0),
            (2.0, Join::Miter, Cap::Square, 0.5, SQRT_2),
            (2.0, Join::Bevel, Cap::Round, 4.0, 1.0),
            (-2.0, Join::Round, Cap::Butt, 4.0, 0.0),
        ];
        for (width, join, cap, limit, reach) in pens {
            let pen = Stroke::new(width).with_join(join).with_cap(cap);
            let pen = pen.with_miter_limit(limit);
            let mut scene = Scene::new();
            scene.stroke(bend.clone(), pen, Color::BLACK);
            let bounds = scene.bounds().unwrap();
            let room = Rect::new(-reach, -reach, 10.0 + reach, 10.0 + reach);
            assert_eq!(bounds, room, "{pen:?}");
            // And the room is enough for what the pen really leaves.
            let left = bend.stroke(pen, 0.01).bounds().unwrap_or(bounds);
            assert_eq!(left.union(bounds), bounds, "{pen:?}");
        }
    }

    #[test]
    fn the_bounds_follow_a_scene_to_where_it_is_placed() {
        let square = |x: f64, y: f64| Path::rect(Rect::new(x, y, x + 10.0, y + 10.0));
        let flat: Path = "M50 50 H90".parse().unwrap();
        let mut content = Scene::new();
        // A path with no area adds nothing, a second shape adds itself.
        for path in [square(0.0, 0.0), flat.clone(), square(20.0, 5.0)] {
            content.fill(path, FillRule::NonZero, Color::BLACK);
        }
        assert_eq!(content.bounds(), Some(Rect::new(0.0, 0.0, 30.0, 15.0)));
        let content = Arc::new(content);

        // All of it, the part a clip leaves, and nothing where the clip
        // misses, each moved to where the group puts it.
        let moved = Affine::translate(Vector::new(100.0, 0.0)) * Affine::scale(2.0, 3.0);
        let cases = [
            (None, Some([100.0, 0.0, 160.0, 45.0])),
            (Some((25.0, 10.0)), Some([150.0, 30.0, 160.0, 45.0])),
            (Some((40.0, 0.0)), None),
        ];
        let mut group = Group::default().with_transform(moved);
        for (clip, bounds) in cases {
            group.clip = clip.map(|(x, y)| square(x, y));
            let mut scene = Scene::new();
            scene.place(group.clone(), content.clone());
            let bounds = bounds.map(|[x0, y0, x1, y1]| Rect::new(x0, y0, x1, y1));
            assert_eq!(scene.bounds(), bounds, "{:?}", group.clip);
        }
        let built = group.with_clip(flat.clone()).with_opacity(0.25);
        let parts = (built.transform, built.clip, built.opacity);
        assert_eq!(parts, (moved, Some(flat), 0.25));
    }
}
