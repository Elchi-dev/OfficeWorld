//! Colours as the eye tells them apart: Oklab.

use ow_motion::Animatable;

use crate::srgb::Color;

/// A colour in Oklab, a space built so that equal steps look like equal
/// steps: a lightness, and two axes of colour, green to red and blue to
/// yellow.
///
/// It is the space to blend, animate and build palettes in. Half way from
/// one colour to another here looks half way, where in sRGB it is often a
/// darker, greyer colour than either end. CSS writes it as `oklab()` and,
/// with chroma and hue for the two axes, as `oklch()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    /// From 0 for black to 1 for white.
    pub lightness: f32,
    /// From green, below zero, to red, above.
    pub a: f32,
    /// From blue, below zero, to yellow, above.
    pub b: f32,
    /// How opaque the colour is: 0 lets everything through, 1 nothing.
    pub alpha: f32,
}

/// How far outside 0 to 1 an amount of light may be and still count as a
/// colour a screen can show. The way there and back is not exact, and white
/// itself comes out a few ten-millionths above 1.
const GAMUT_SLACK: f64 = 1e-5;

impl Oklab {
    /// A colour from its lightness, its two colour axes and its alpha.
    pub const fn new(lightness: f32, a: f32, b: f32, alpha: f32) -> Self {
        Self {
            lightness,
            a,
            b,
            alpha,
        }
    }

    /// A colour from its lightness, its chroma (how colourful it is, 0 for
    /// grey) and its hue in degrees, as `oklch()` in CSS takes them.
    pub fn from_lch(lightness: f32, chroma: f32, hue: f32, alpha: f32) -> Self {
        let (sin, cos) = hue.to_radians().sin_cos();
        Self::new(lightness, chroma * cos, chroma * sin, alpha)
    }

    /// How colourful the colour is: 0 for a grey.
    pub fn chroma(self) -> f32 {
        self.a.hypot(self.b)
    }

    /// The hue in degrees, from 0 up to 360. About 30 is red, 140 green and
    /// 265 blue.
    pub fn hue(self) -> f32 {
        self.b.atan2(self.a).to_degrees().rem_euclid(360.0)
    }

    /// The colour a share `t` of the way to another: this colour at 0, the
    /// other at 1.
    #[must_use]
    pub fn mix(self, other: Self, t: f32) -> Self {
        self.plus(other.minus(self).scaled(f64::from(t)))
    }

    /// The amounts of red, green and blue light the colour stands for. They
    /// leave 0 to 1 for a colour no sRGB screen can show.
    fn to_linear(self) -> [f64; 3] {
        let (lightness, a, b) = (
            f64::from(self.lightness),
            f64::from(self.a),
            f64::from(self.b),
        );
        let long = (lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
        let medium = (lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
        let short = (lightness - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);
        [
            4.076_741_662_1 * long - 3.307_711_591_3 * medium + 0.230_969_929_2 * short,
            -1.268_438_004_6 * long + 2.609_757_401_1 * medium - 0.341_319_396_5 * short,
            -0.004_196_086_3 * long - 0.703_418_614_7 * medium + 1.707_614_701_0 * short,
        ]
    }

    /// Whether an sRGB screen can show the colour.
    pub fn in_gamut(self) -> bool {
        self.to_linear()
            .iter()
            .all(|light| (-GAMUT_SLACK..=1.0 + GAMUT_SLACK).contains(light))
    }

    /// The nearest colour an sRGB screen can show: the same lightness and
    /// hue, with as much of the chroma as fits.
    ///
    /// Cutting the red, green and blue parts off at 0 and 1 would also give
    /// a colour that can be shown, and it would be another hue: a strong
    /// blue turns purple that way.
    #[must_use]
    pub fn to_gamut(self) -> Self {
        if self.in_gamut() {
            return self;
        }
        let lightness = self.lightness.clamp(0.0, 1.0);
        let (mut fits, mut too_much) = (0.0, self.chroma());
        // Halving 24 times finds the chroma to a ten-millionth of where it
        // began, which is finer than an f32 holds.
        for _ in 0..24 {
            let chroma = f32::midpoint(fits, too_much);
            if Self::from_lch(lightness, chroma, self.hue(), self.alpha).in_gamut() {
                fits = chroma;
            } else {
                too_much = chroma;
            }
        }
        Self::from_lch(lightness, fits, self.hue(), self.alpha)
    }
}

impl From<Color> for Oklab {
    fn from(colour: Color) -> Self {
        let [red, green, blue] = colour.to_linear();
        let long =
            (0.412_221_470_8 * red + 0.536_332_536_3 * green + 0.051_445_992_9 * blue).cbrt();
        let medium =
            (0.211_903_498_2 * red + 0.680_699_545_1 * green + 0.107_396_956_6 * blue).cbrt();
        let short =
            (0.088_302_461_9 * red + 0.281_718_837_6 * green + 0.629_978_700_5 * blue).cbrt();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a colour part is stored as f32"
        )]
        let part = |value: f64| value as f32;
        Self::new(
            part(0.210_454_255_3 * long + 0.793_617_785_0 * medium - 0.004_072_046_8 * short),
            part(1.977_998_495_1 * long - 2.428_592_205_0 * medium + 0.450_593_709_9 * short),
            part(0.025_904_037_1 * long + 0.782_771_766_2 * medium - 0.808_675_766_0 * short),
            colour.alpha,
        )
    }
}

/// The sRGB colour for an Oklab one. Its parts leave 0 to 1 when no screen
/// can show the colour: call [`Oklab::to_gamut`] first to keep the hue.
impl From<Oklab> for Color {
    fn from(colour: Oklab) -> Self {
        Self::from_linear(colour.to_linear()).with_alpha(colour.alpha)
    }
}

impl Animatable for Oklab {
    fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    fn plus(self, other: Self) -> Self {
        Self::new(
            self.lightness + other.lightness,
            self.a + other.a,
            self.b + other.b,
            self.alpha + other.alpha,
        )
    }

    fn scaled(self, factor: f64) -> Self {
        let [lightness, a, b, alpha] = [self.lightness, self.a, self.b, self.alpha].scaled(factor);
        Self::new(lightness, a, b, alpha)
    }

    fn length_squared(self) -> f64 {
        [self.lightness, self.a, self.b, self.alpha].length_squared()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);

    #[test]
    fn the_primaries_land_where_the_css_specification_puts_them() {
        // The values CSS Color 4 and its reference implementation give for
        // the corners of sRGB. They pin all eighteen constants of the way
        // there: one wrong digit moves one of these.
        let expected = [
            (RED, [0.627_955, 0.224_863, 0.125_846]),
            (GREEN, [0.866_440, -0.233_888, 0.179_498]),
            (BLUE, [0.452_014, -0.032_457, -0.311_528]),
            (Color::WHITE, [1.0, 0.0, 0.0]),
            (Color::BLACK, [0.0, 0.0, 0.0]),
        ];
        for (colour, [lightness, a, b]) in expected {
            let lab = Oklab::from(colour);
            let off = (lab.lightness - lightness).abs() + (lab.a - a).abs() + (lab.b - b).abs();
            assert!(off < 2e-5, "{colour:?} became {lab:?}");
        }
        // As oklch(), red is written with a chroma of 0.2577 and a hue of
        // 29.23 degrees.
        let red = Oklab::from(RED);
        assert!((red.chroma() - 0.2577).abs() < 1e-4 && (red.hue() - 29.23).abs() < 0.01);
        assert!((Oklab::from(BLUE).hue() - 264.05).abs() < 0.01);
    }

    #[test]
    fn every_colour_comes_back_from_oklab_as_itself() {
        for step in 0..=255_u8 {
            // A walk through the cube that touches every value of each part.
            let (green, alpha) = (step.wrapping_mul(7), step.wrapping_mul(13));
            let bytes = [step, green, 255 - step, alpha];
            let there_and_back = Color::from(Oklab::from(Color::from_rgba8(bytes)));
            assert_eq!(there_and_back.to_rgba8(), bytes);
        }
    }

    #[test]
    fn half_way_in_oklab_looks_half_way() {
        // Red to blue. In sRGB the middle is a dark purple, (128, 0, 128).
        // In Oklab it keeps the lightness between the two. Worked out apart
        // from this crate.
        let middle = Oklab::from(RED).mix(Oklab::from(BLUE), 0.5);
        assert_eq!(Color::from(middle).to_rgba8(), [140, 83, 162, 255]);
        let quarter = Oklab::new(0.2, 0.1, -0.1, 1.0).mix(Oklab::new(0.6, -0.1, 0.3, 0.0), 0.25);
        let off = quarter.minus(Oklab::new(0.3, 0.05, 0.0, 0.75));
        assert!(off.length_squared() < 1e-12, "{quarter:?}");
    }

    #[test]
    fn a_colour_no_screen_can_show_keeps_its_hue() {
        // A green more colourful than sRGB has.
        let wanted = Oklab::from_lch(0.7, 0.4, 150.0, 0.5);
        assert!(!wanted.in_gamut() && Oklab::from(Color::WHITE).in_gamut());
        let shown = wanted.to_gamut();
        assert!(shown.in_gamut());
        assert!((shown.hue() - 150.0).abs() < 0.01 && (shown.lightness, shown.alpha) == (0.7, 0.5));
        // As much chroma as fits: a hundredth more no longer does.
        assert!(shown.chroma() < 0.4);
        assert!(!Oklab::from_lch(0.7, shown.chroma() * 1.01, 150.0, 0.5).in_gamut());
        // Lighter than white and darker than black end as white and black.
        let shown_as = |colour: Oklab| Color::from(colour.to_gamut()).to_rgba8();
        assert_eq!(shown_as(Oklab::new(1.4, 0.1, 0.0, 1.0)), [255; 4]);
        assert_eq!(shown_as(Oklab::new(-0.2, 0.0, 0.1, 1.0)), [0, 0, 0, 255]);
        let fine = Oklab::from(BLUE);
        assert_eq!(fine.to_gamut(), fine);
    }

    #[test]
    fn a_spring_moves_all_four_parts() {
        let colour = Oklab::new(0.5, 0.1, -0.2, 1.0);
        assert_eq!(colour.plus(colour.scaled(-1.0)), Oklab::zero());
        assert_eq!(Oklab::new(1.0, 2.0, 2.0, 4.0).length_squared(), 25.0);
    }
}
