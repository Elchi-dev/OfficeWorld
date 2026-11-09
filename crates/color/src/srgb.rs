//! Colours as files and screens give them: sRGB.

use std::error::Error;
use std::fmt;

/// A colour in sRGB, the space of screens, of CSS and of the office file
/// formats, with an alpha that says how much of what lies behind shows
/// through.
///
/// The parts run from 0 to 1 and are the encoded values, the ones a file
/// stores, not amounts of light. The colour is not multiplied by its alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// The red part.
    pub red: f32,
    /// The green part.
    pub green: f32,
    /// The blue part.
    pub blue: f32,
    /// How opaque the colour is: 0 lets everything through, 1 nothing.
    pub alpha: f32,
}

/// A text that is not a colour in hexadecimal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidHex(String);

impl fmt::Display for InvalidHex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\"{}\" is not a colour. Write 3, 4, 6 or 8 hexadecimal digits, with or \
             without a # in front, such as #1a73e8.",
            self.0
        )
    }
}

impl Error for InvalidHex {}

impl Color {
    /// Black.
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    /// White.
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    /// Nothing at all: black that lets everything through.
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// An opaque colour from its red, green and blue part.
    pub const fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::rgba(red, green, blue, 1.0)
    }

    /// A colour from its red, green, blue and alpha part.
    pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// A colour from four bytes: red, green, blue and alpha.
    pub fn from_rgba8([red, green, blue, alpha]: [u8; 4]) -> Self {
        let part = |byte: u8| f32::from(byte) / 255.0;
        Self::rgba(part(red), part(green), part(blue), part(alpha))
    }

    /// The colour as four bytes: red, green, blue and alpha. A part outside
    /// 0 to 1 becomes the nearest end, and one that is not a number
    /// becomes 0.
    pub fn to_rgba8(self) -> [u8; 4] {
        [self.red, self.green, self.blue, self.alpha].map(|part| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "the value is clamped to 0..=255 first, and a cast of not-a-number gives 0"
            )]
            let byte = (part.clamp(0.0, 1.0) * 255.0).round() as u8;
            byte
        })
    }

    /// A colour written in hexadecimal, as CSS and the office formats write
    /// it: `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa`, with or without the
    /// `#`.
    ///
    /// # Errors
    ///
    /// Refuses a text of another length or with a character that is not a
    /// hexadecimal digit.
    pub fn from_hex(text: &str) -> Result<Self, InvalidHex> {
        let invalid = || InvalidHex(text.to_owned());
        let digits = text.strip_prefix('#').unwrap_or(text);
        let values = digits
            .chars()
            .map(|digit| digit.to_digit(16))
            .collect::<Option<Vec<u32>>>()
            .ok_or_else(invalid)?;
        // One digit stands for the same digit twice: f is ff.
        let pair = |high: u32, low: u32| u8::try_from(high * 16 + low).unwrap_or(u8::MAX);
        let bytes: Vec<u8> = match values.len() {
            3 | 4 => values.iter().map(|&digit| pair(digit, digit)).collect(),
            6 | 8 => values
                .chunks(2)
                .map(|both| pair(both[0], both[1]))
                .collect(),
            _ => return Err(invalid()),
        };
        let alpha = bytes.get(3).copied().unwrap_or(u8::MAX);
        Ok(Self::from_rgba8([bytes[0], bytes[1], bytes[2], alpha]))
    }

    /// The same colour with another alpha.
    #[must_use]
    pub const fn with_alpha(self, alpha: f32) -> Self {
        Self::rgba(self.red, self.green, self.blue, alpha)
    }

    /// The colour a share `t` of the way to another, part by part in sRGB:
    /// this colour at 0, the other at 1.
    ///
    /// That is how CSS and the office formats blend the stops of a
    /// gradient, so it is what a file's gradient has to be drawn with. For
    /// a blend that looks even to the eye, mix in
    /// [`Oklab`](crate::Oklab).
    #[must_use]
    pub fn mix(self, other: Self, t: f32) -> Self {
        let part = |from: f32, to: f32| from + (to - from) * t;
        Self::rgba(
            part(self.red, other.red),
            part(self.green, other.green),
            part(self.blue, other.blue),
            part(self.alpha, other.alpha),
        )
    }

    /// The colour seen when this one lies over a backdrop.
    #[must_use]
    pub fn over(self, backdrop: Self) -> Self {
        let showing_through = backdrop.alpha * (1.0 - self.alpha);
        let alpha = self.alpha + showing_through;
        if alpha <= 0.0 {
            return Self::TRANSPARENT;
        }
        let part = |top: f32, below: f32| (top * self.alpha + below * showing_through) / alpha;
        Self::rgba(
            part(self.red, backdrop.red),
            part(self.green, backdrop.green),
            part(self.blue, backdrop.blue),
            alpha,
        )
    }

    /// The amounts of red, green and blue light the colour stands for, each
    /// from 0 to 1.
    pub fn to_linear(self) -> [f64; 3] {
        [self.red, self.green, self.blue].map(|part| decode(f64::from(part)))
    }

    /// The opaque colour that stands for these amounts of red, green and
    /// blue light.
    pub fn from_linear(light: [f64; 3]) -> Self {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a colour part is stored as f32"
        )]
        let [red, green, blue] = light.map(|amount| encode(amount) as f32);
        Self::rgb(red, green, blue)
    }

    /// How bright the colour is to the eye, from 0 for black to 1 for
    /// white, as the accessibility guidelines for the web define it.
    pub fn luminance(self) -> f64 {
        let [red, green, blue] = self.to_linear();
        0.2126 * red + 0.7152 * green + 0.0722 * blue
    }

    /// The contrast between two colours, from 1 for none to 21 for black on
    /// white. The accessibility guidelines ask for 4.5 between a text and
    /// its background.
    pub fn contrast(self, other: Self) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
}

/// From a stored sRGB value to an amount of light.
fn decode(encoded: f64) -> f64 {
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

/// From an amount of light to the sRGB value that stores it.
fn encode(light: f64) -> f64 {
    if light <= 0.003_130_8 {
        light * 12.92
    } else {
        1.055 * light.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(text: &str) -> Option<[u8; 4]> {
        Color::from_hex(text).ok().map(Color::to_rgba8)
    }

    #[test]
    fn hexadecimal_is_read_in_every_length_css_allows() {
        assert_eq!(bytes("#1a73e8"), Some([0x1a, 0x73, 0xe8, 0xff]));
        assert_eq!(bytes("1A73E880"), Some([0x1a, 0x73, 0xe8, 0x80]));
        // A short form doubles each digit: #f80 is #ff8800 and not #f08000.
        assert_eq!(bytes("#f80"), Some([0xff, 0x88, 0x00, 0xff]));
        assert_eq!(bytes("f808"), Some([0xff, 0x88, 0x00, 0x88]));
        for wrong in "|#|#12|#12345|#1234567|#123456789|#12345g".split('|') {
            assert_eq!(bytes(wrong), None, "{wrong}");
        }
        let message = Color::from_hex("red").unwrap_err().to_string();
        assert!(message.starts_with("\"red\" is not a colour. Write 3, 4, 6 or 8"));
    }

    #[test]
    fn bytes_survive_the_way_through_a_colour_and_through_light() {
        for byte in 0..=255 {
            let all = [byte, 255 - byte, 7, byte];
            let colour = Color::from_rgba8(all);
            let through_light = Color::from_linear(colour.to_linear()).with_alpha(colour.alpha);
            assert_eq!((colour.to_rgba8(), through_light.to_rgba8()), (all, all));
        }
        // Parts that have left the range, by an overshooting animation or a
        // bad file, end at the nearest byte and do not wrap around.
        let wild = Color::rgba(1.7, -0.3, f32::NAN, 0.5);
        assert_eq!(wild.to_rgba8(), [255, 0, 0, 128]);
    }

    #[test]
    fn light_follows_the_srgb_curve() {
        // Worked out apart from this crate from the definition in IEC
        // 61966-2-1: half the stored value is about a fifth of the light.
        // The second value sits on the joint of the curve's two pieces.
        let light = Color::rgb(0.5, 0.04045, 10.0 / 255.0).to_linear();
        let expected = [0.214_041_140_482, 0.003_130_804_954, 0.003_035_269_835];
        for (got, wanted) in light.iter().zip(expected) {
            assert!((got - wanted).abs() < 1e-9, "{light:?}");
        }
    }

    #[test]
    fn contrast_is_the_one_the_guidelines_measure() {
        let on_white = |hex| Color::from_hex(hex).unwrap().contrast(Color::WHITE);
        // What the contrast checkers of the web print for these colours on
        // white. #777 just fails the 4.5 the guidelines ask for and #767676
        // just passes; the three primaries pin the weight of each part.
        let expected = [
            ("#000", 21.0),
            ("#777777", 4.478_09),
            ("#767676", 4.542_22),
            ("#f00", 3.998_48),
            ("#0f0", 1.372_19),
            ("#00f", 8.592_47),
        ];
        for (hex, contrast) in expected {
            assert!(
                (on_white(hex) - contrast).abs() < 1e-4,
                "{hex}: {}",
                on_white(hex)
            );
        }
        assert!((Color::WHITE.contrast(Color::BLACK) - 21.0).abs() < 1e-9);
    }

    #[test]
    fn mixing_and_layering() {
        let (red, blue) = (Color::rgb(1.0, 0.0, 0.0), Color::rgba(0.0, 0.0, 1.0, 0.5));
        assert_eq!(red.mix(blue, 0.25), Color::rgba(0.75, 0.0, 0.25, 0.875));

        // Half transparent blue over opaque red: half of each, and opaque.
        assert_eq!(blue.over(red), Color::rgb(0.5, 0.0, 0.5));
        // Over a half transparent backdrop the top colour weighs more: it
        // covers half, and the backdrop half of what is left.
        let stacked = blue.over(Color::rgba(1.0, 0.5, 0.0, 0.5));
        let expected = Color::rgba(1.0 / 3.0, 1.0 / 6.0, 2.0 / 3.0, 0.75);
        assert_eq!(stacked.mix(expected, 0.5).to_rgba8(), expected.to_rgba8());
        assert!((stacked.red - expected.red).abs() + (stacked.green - expected.green).abs() < 1e-6);
        assert_eq!(Color::TRANSPARENT.over(red), red);
        // Nothing over nothing is nothing, not zero divided by zero.
        let nothing = Color::WHITE.with_alpha(0.0);
        assert_eq!(nothing.over(Color::TRANSPARENT), Color::TRANSPARENT);
    }
}
