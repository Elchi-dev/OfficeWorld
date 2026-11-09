//! Ramps: the colours that everything of one tint is drawn with.

use ow_color::{Color, Oklab};

/// Whether the interface is light with dark text, or dark with light text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scheme {
    /// Dark text on light surfaces.
    #[default]
    Light,
    /// Light text on dark surfaces.
    Dark,
}

/// What a ramp is grown from: a hue, how colourful it is, and how light its
/// solid colour is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tint {
    /// The hue in degrees, as `oklch()` in CSS counts them: about 30 is
    /// red, 140 green and 265 blue.
    pub hue: f32,
    /// How colourful the solid colour is. A screen shows about 0.2 for most
    /// hues, and 0 is grey.
    pub chroma: f32,
    /// How light the solid colour is, from 0 to 1.
    pub lightness: f32,
}

impl Tint {
    /// A grey with a breath of blue: what most of an interface is made of.
    pub const NEUTRAL: Self = Self::new(258.0, 0.012, 0.58);
    /// Blue.
    pub const BLUE: Self = Self::new(258.0, 0.19, 0.60);
    /// Green.
    pub const GREEN: Self = Self::new(150.0, 0.16, 0.63);
    /// Orange.
    pub const ORANGE: Self = Self::new(55.0, 0.17, 0.70);
    /// Violet.
    pub const VIOLET: Self = Self::new(300.0, 0.20, 0.58);
    /// Red.
    pub const RED: Self = Self::new(25.0, 0.21, 0.60);
    /// Teal.
    pub const TEAL: Self = Self::new(195.0, 0.11, 0.62);
    /// Yellow.
    pub const YELLOW: Self = Self::new(95.0, 0.16, 0.84);

    /// A tint from its hue, its chroma and the lightness of its solid
    /// colour.
    pub const fn new(hue: f32, chroma: f32, lightness: f32) -> Self {
        Self {
            hue,
            chroma,
            lightness,
        }
    }

    /// The tint of a colour somebody picked: a ramp grown from it has that
    /// colour, or the nearest one that text can be read on, as its solid.
    pub fn of(color: Color) -> Self {
        let lab = Oklab::from(color);
        Self::new(lab.hue(), lab.chroma(), lab.lightness)
    }
}

/// The colours of one tint: for each thing an interface draws, the colour
/// it is drawn in.
///
/// A ramp is not picked by eye. It is worked out in Oklab from its tint, so
/// ramps of different hues have the same steps of lightness and look like
/// one family, and so that text can be read on it: every promise in the
/// documentation of a field holds for every tint in both schemes, and the
/// tests measure each one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ramp {
    /// What lies behind everything: the window.
    pub backdrop: Color,
    /// Panels, cards and sheets that lie on the backdrop.
    pub surface: Color,
    /// A control at rest: a button, a field, a selected row.
    pub fill: Color,
    /// The same control under the pointer.
    pub fill_hover: Color,
    /// The same control while it is pressed.
    pub fill_pressed: Color,
    /// A line that only separates.
    pub line_soft: Color,
    /// The edge of a control.
    pub line: Color,
    /// An edge that has to be seen to use the interface, such as that of an
    /// empty text field. It stands out from the backdrop by at least 3 to
    /// 1, which is what the accessibility guidelines ask of such an edge.
    pub line_strong: Color,
    /// The tint itself: a filled button, a selection, a badge.
    pub solid: Color,
    /// The solid colour under the pointer.
    pub solid_hover: Color,
    /// Text and icons on the solid colour, at 4.5 to 1 or more on both the
    /// solid and its hover colour.
    pub on_solid: Color,
    /// Text that steps back: labels, hints, the second line. At least 4.5
    /// to 1 on the backdrop, on a surface and on a fill.
    pub text_soft: Color,
    /// Text. At least 7 to 1 on everything up to a pressed fill.
    pub text: Color,
}

/// The lightness and the share of the tint's chroma of the seven colours
/// from the backdrop to the edge of a control, on a light scheme.
const LIGHT: [(f32, f32); 7] = [
    (0.992, 0.03),
    (0.978, 0.07),
    (0.951, 0.16),
    (0.925, 0.25),
    (0.897, 0.34),
    (0.862, 0.44),
    (0.812, 0.57),
];

/// The same on a dark scheme.
const DARK: [(f32, f32); 7] = [
    (0.176, 0.08),
    (0.208, 0.12),
    (0.25, 0.25),
    (0.283, 0.34),
    (0.317, 0.42),
    (0.358, 0.5),
    (0.413, 0.6),
];

/// A solid colour lighter than this gets dark text on it, and a darker one
/// white text.
const WHITE_TEXT_UP_TO: f32 = 0.66;

/// The lightness nearest to `from`, on the way to `to`, at which `passes`
/// holds, where it holds from some lightness on all the way to `to`.
fn reach(from: f32, to: f32, passes: impl Fn(f32) -> bool) -> f32 {
    if passes(from) {
        return from;
    }
    let (mut fails, mut holds) = (from, to);
    for _ in 0..20 {
        let middle = f32::midpoint(fails, holds);
        if passes(middle) {
            holds = middle;
        } else {
            fails = middle;
        }
    }
    holds
}

impl Ramp {
    /// The ramp of a tint for a scheme.
    pub fn new(tint: Tint, scheme: Scheme) -> Self {
        // A colour of the tint's hue at a lightness, with a share of its
        // chroma, or as much of that as a screen can show.
        let shade = |lightness: f32, share: f32| {
            let wanted = Oklab::from_lch(lightness, tint.chroma * share, tint.hue, 1.0);
            Color::from(wanted.to_gamut())
        };
        // Text is at the far end from the backdrop: towards black on a
        // light scheme and towards white on a dark one.
        let (steps, far, text, soft_share) = match scheme {
            Scheme::Light => (LIGHT, 0.0, shade(0.27, 0.45), 0.85),
            Scheme::Dark => (DARK, 1.0, shade(0.935, 0.25), 0.75),
        };
        let [
            backdrop,
            surface,
            fill,
            fill_hover,
            fill_pressed,
            line_soft,
            line,
        ] = steps.map(|(lightness, share)| shade(lightness, share));
        let stands_out = |lightness: f32| shade(lightness, 0.75).contrast(backdrop) >= 3.0;
        let strong = reach(steps[6].0, far, stands_out);
        let readable = |lightness: f32| shade(lightness, soft_share).contrast(fill) >= 4.5;
        let soft = reach(strong, far, readable);
        // The solid colour is the tint's own, moved away from the colour
        // of the text on it until that text can be read. Under the pointer
        // it moves on the same way, so the text only gets clearer.
        let (on_solid, away, step) = if tint.lightness <= WHITE_TEXT_UP_TO {
            (Color::WHITE, 0.0, -0.04)
        } else {
            (shade(0.2, 0.1), 1.0, 0.04)
        };
        let carries = |lightness: f32| on_solid.contrast(shade(lightness, 1.0)) >= 4.5;
        let solid = reach(tint.lightness, away, carries);
        Self {
            backdrop,
            surface,
            fill,
            fill_hover,
            fill_pressed,
            line_soft,
            line,
            line_strong: shade(strong, 0.75),
            solid: shade(solid, 1.0),
            solid_hover: shade(solid + step, 1.0),
            on_solid,
            text_soft: shade(soft, soft_share),
            text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINTS: [Tint; 8] = [
        Tint::NEUTRAL,
        Tint::BLUE,
        Tint::GREEN,
        Tint::ORANGE,
        Tint::VIOLET,
        Tint::RED,
        Tint::TEAL,
        Tint::YELLOW,
    ];

    fn colors(ramp: Ramp) -> [Color; 13] {
        [
            ramp.backdrop,
            ramp.surface,
            ramp.fill,
            ramp.fill_hover,
            ramp.fill_pressed,
            ramp.line_soft,
            ramp.line,
            ramp.line_strong,
            ramp.solid,
            ramp.solid_hover,
            ramp.on_solid,
            ramp.text_soft,
            ramp.text,
        ]
    }

    #[test]
    fn every_ramp_keeps_what_its_fields_promise() {
        for (tint, scheme) in TINTS
            .iter()
            .flat_map(|tint| [(tint, Scheme::Light), (tint, Scheme::Dark)])
        {
            let ramp = Ramp::new(*tint, scheme);
            let on = |text: Color, grounds: &[Color]| {
                let worst = grounds.iter().map(|ground| text.contrast(*ground));
                worst.fold(f64::INFINITY, f64::min)
            };
            let soft = on(ramp.text_soft, &[ramp.backdrop, ramp.surface, ramp.fill]);
            let text = on(ramp.text, &[ramp.backdrop, ramp.fill_pressed]);
            let solid = on(ramp.on_solid, &[ramp.solid, ramp.solid_hover]);
            let edge = ramp.line_strong.contrast(ramp.backdrop);
            let kept = soft >= 4.5 && text >= 7.0 && solid >= 4.5 && edge >= 3.0;
            assert!(kept, "{tint:?} {scheme:?}: {soft} {text} {solid} {edge}");
            // No darker and no further from the tint than the promise
            // needs: a colour is moved until it holds, and not beyond.
            assert!(
                soft < 4.56 && edge < 3.04,
                "{tint:?} {scheme:?}: {soft} {edge}"
            );
            // From the backdrop to the strong edge, each colour is further
            // from the backdrop than the one before.
            let steps = &colors(ramp)[..8];
            let further = |pair: &[Color]| {
                (pair[0].luminance() > pair[1].luminance()) == (scheme == Scheme::Light)
            };
            assert!(steps.windows(2).all(further), "{tint:?} {scheme:?}");
        }
    }

    #[test]
    fn the_ramps_are_the_ones_worked_out_apart() {
        // The same recipe written a second time, in Python with 64-bit
        // numbers, gave these. A step of a byte may differ where the two
        // round differently.
        let expected = [
            (
                Tint::BLUE,
                Scheme::Light,
                "fbfdff f3f8ff e5f0ff d8e8ff c9dfff b6d4ff 9bc4ff 5994ea 1f73e5 0d67d8 ffffff 296bca 07254f",
            ),
            (
                Tint::NEUTRAL,
                Scheme::Dark,
                "101111 171818 212223 282a2c 303235 3b3d40 494b4f 5d6065 72777e 676c72 ffffff 85888d e8e9eb",
            ),
            (
                Tint::YELLOW,
                Scheme::Light,
                "fdfcf9 faf8f0 f4efdc eee7c9 e8ddb5 e0d29d d4c17c aa922c ebc831 f9d544 18160e 816b00 2f2600",
            ),
        ];
        for (tint, scheme, hexes) in expected {
            let ramp = colors(Ramp::new(tint, scheme));
            for (color, hex) in ramp.iter().zip(hexes.split(' ')) {
                let (got, wanted) = (color.to_rgba8(), Color::from_hex(hex).unwrap().to_rgba8());
                let near = |part: usize| got[part].abs_diff(wanted[part]) <= 1;
                assert!((0..4).all(near), "{tint:?} {scheme:?}: {got:?} for {hex}");
            }
        }
        // A ramp grown from a picked colour has that colour as its solid.
        let picked = Color::from_hex("1a5fd0").unwrap();
        let grown = Ramp::new(Tint::of(picked), Scheme::Dark);
        assert_eq!(grown.solid.to_rgba8(), picked.to_rgba8());
    }
}
