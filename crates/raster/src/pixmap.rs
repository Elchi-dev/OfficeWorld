//! Pictures in memory, and drawing into them.

use std::error::Error;
use std::fmt;

use ow_color::Color;
use ow_path::{FillRule, Path, Stroke};

use crate::mask::{Mask, TOLERANCE};

/// The most pixels a picture may have: a quarter of a billion, which is a
/// gibibyte of memory. A larger picture is drawn in parts.
const MOST_PIXELS: u64 = 1 << 28;

/// A picture that was asked for and is too large to hold in memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooLarge {
    /// The width that was asked for, in pixels.
    pub width: u32,
    /// The height that was asked for, in pixels.
    pub height: u32,
}

impl fmt::Display for TooLarge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (width, height) = (self.width, self.height);
        write!(f, "A picture of {width} by {height} pixels is too large.")
    }
}

impl Error for TooLarge {}

/// A picture in memory: rows of pixels, each a red, a green, a blue and an
/// alpha byte.
///
/// The colours are sRGB, as files and screens give them, and each is stored
/// multiplied by its alpha. In that form laying one colour over another is
/// one step for each byte. Colours are blended as the numbers they are
/// stored as and not as amounts of light. That is how browsers and the
/// office suites blend, so a half transparent shape looks here as it does
/// there.
///
/// ```
/// use ow_color::Color;
/// use ow_path::{FillRule, Path, Stroke};
/// use ow_raster::Pixmap;
///
/// let mut picture = Pixmap::new(64, 64)?;
/// let roof: Path = "M8 40 L32 12 L56 40 Z".parse()?;
/// picture.fill(&roof, FillRule::NonZero, Color::from_hex("#c2410c")?);
/// picture.stroke(&roof, Stroke::new(4.0), Color::BLACK);
///
/// // Red, green, blue and alpha: inside the roof, on its lower edge, and
/// // beside it, where nothing was drawn.
/// assert_eq!(picture.pixel(32, 30), Some([194, 65, 12, 255]));
/// assert_eq!(picture.pixel(32, 40), Some([0, 0, 0, 255]));
/// assert_eq!(picture.pixel(8, 12), Some([0, 0, 0, 0]));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixmap {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

impl Pixmap {
    /// A picture of a size, transparent everywhere.
    ///
    /// # Errors
    ///
    /// Refuses a picture of more than a quarter of a billion pixels, and
    /// one there is no memory for.
    pub fn new(width: u32, height: u32) -> Result<Self, TooLarge> {
        let too_large = TooLarge { width, height };
        let pixels = u64::from(width) * u64::from(height);
        let fitting = Some(pixels).filter(|pixels| *pixels <= MOST_PIXELS);
        let length = fitting.and_then(|pixels| usize::try_from(pixels * 4).ok());
        let length = length.ok_or(too_large)?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length).map_err(|_| too_large)?;
        bytes.resize(length, 0);
        Ok(Self {
            width,
            height,
            bytes,
        })
    }

    /// The width in pixels.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// The height in pixels.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The picture as it is stored: row by row from the top, four bytes to
    /// a pixel, the colour multiplied by its alpha.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The four bytes of a pixel as they are stored, or `None` for a pixel
    /// outside the picture.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        let inside = x < self.width && y < self.height;
        let start = inside.then(|| (y as usize * self.width as usize + x as usize) * 4)?;
        self.bytes[start..start + 4].try_into().ok()
    }

    /// The picture as files store it: four bytes to a pixel, the colour not
    /// multiplied by its alpha.
    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut bytes = self.bytes.clone();
        for pixel in bytes.as_chunks_mut::<4>().0 {
            // A pixel that lets everything through has no colour to bring
            // back. Its division gives no number, and that becomes 0.
            let alpha = f32::from(pixel[3]);
            for part in &mut pixel[..3] {
                *part = to_byte(f32::from(*part) * 255.0 / alpha);
            }
        }
        bytes
    }

    /// Fills a path with a colour. One unit of the path is one pixel.
    pub fn fill(&mut self, path: &Path, rule: FillRule, color: Color) {
        self.paint(&Mask::new(path, rule, self.width, self.height), color);
    }

    /// Draws along a path with a pen.
    pub fn stroke(&mut self, path: &Path, pen: Stroke, color: Color) {
        let outline = path.stroke(pen, TOLERANCE);
        self.fill(&outline, FillRule::NonZero, color);
    }

    /// Lays a colour over the picture, as much of it on each pixel as a
    /// mask made for this picture says is covered.
    fn paint(&mut self, mask: &Mask, color: Color) {
        // A part of a colour outside 0 to 1 becomes the nearest end.
        #[expect(
            clippy::manual_clamp,
            reason = "unlike a clamp, this turns what is not a number into 0"
        )]
        let part = |value: f32| value.max(0.0).min(1.0);
        let alpha = part(color.alpha);
        let laid = [color.red, color.green, color.blue, 1.0].map(|value| part(value) * alpha);
        for (x, y, row) in mask.rows() {
            let start = (y as usize * self.width as usize + x as usize) * 4;
            let pixels = self.bytes[start..].as_chunks_mut::<4>().0;
            for (pixel, &coverage) in pixels.iter_mut().zip(row) {
                let kept = 1.0 - alpha * coverage;
                for (byte, new) in pixel.iter_mut().zip(laid) {
                    *byte = to_byte(new * coverage * 255.0 + f32::from(*byte) * kept);
                }
            }
        }
    }
}

/// The nearest byte to a value from 0 to 255. What is not a number is 0.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value lies between 0 and 255, and a cast of not-a-number gives 0"
)]
fn to_byte(value: f32) -> u8 {
    (value + 0.5) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture 4 pixels wide and 2 high with colours laid over the first
    /// two and a half pixels of its upper row, one after another.
    fn laid(colors: &[Color]) -> Pixmap {
        let mut picture = Pixmap::new(4, 2).unwrap();
        let left: Path = "M0 0 H2.5 V1 H0 Z".parse().unwrap();
        for &color in colors {
            picture.fill(&left, FillRule::NonZero, color);
        }
        picture
    }

    #[test]
    fn a_colour_covers_as_much_of_a_pixel_as_the_shape_does() {
        let picture = laid(&[Color::from_hex("#c2400c").unwrap()]);
        let (whole, half) = ([194, 64, 12, 255], [97, 32, 6, 128]);
        let rows = [[whole, whole, half, [0; 4]], [[0; 4]; 4]];
        assert_eq!(picture.bytes(), rows.as_flattened().as_flattened());
        // A file gets the colour back from under its alpha, as well as
        // half the steps of a byte allow.
        assert_eq!(picture.to_rgba8()[8..16], [193, 64, 12, 128, 0, 0, 0, 0]);
        let asked = [(2, 0), (2, 1), (4, 0), (0, 2)].map(|(x, y)| picture.pixel(x, y));
        assert_eq!(asked, [Some(half), Some([0; 4]), None, None]);
        assert_eq!((picture.width(), picture.height()), (4, 2));
    }

    #[test]
    fn one_colour_over_another_is_what_the_colour_crate_says() {
        // Colours through each other, in the form they are stored in, to
        // the byte. Rounding twice may cost one step.
        let colors = ["#c2410c", "#1a73e880", "#00000000", "#10b981e6"];
        let colors = colors.map(|hex| Color::from_hex(hex).unwrap());
        for (below, above) in colors
            .iter()
            .flat_map(|below| colors.map(|above| (*below, above)))
        {
            let seen = above.over(below);
            let stored = [seen.red, seen.green, seen.blue, 1.0].map(|part| part * seen.alpha);
            let expected = Color::rgba(stored[0], stored[1], stored[2], stored[3]).to_rgba8();
            let pixel = laid(&[below, above]).pixel(0, 0).unwrap();
            let near = |index: usize| pixel[index].abs_diff(expected[index]) <= 1;
            assert!((0..4).all(near), "{above:?} over {below:?}: {pixel:?}");
        }
        // A colour that is no colour changes nothing, and one outside what
        // a byte holds becomes the nearest colour a byte does hold.
        let no_colour = Color::rgba(f32::NAN, 0.5, 0.5, f32::NAN);
        assert_eq!(laid(&[colors[0], no_colour]), laid(&[colors[0]]));
        let beyond = Color::rgba(7.0, -1.0, 0.0, 3.0);
        assert_eq!(laid(&[beyond]).pixel(0, 0), Some([255, 0, 0, 255]));
    }

    #[test]
    fn a_picture_too_large_to_hold_is_refused() {
        let too_large = Pixmap::new(1 << 14, (1 << 14) + 1).unwrap_err();
        assert_eq!((too_large.width, too_large.height), (16384, 16385));
        let message = too_large.to_string();
        assert_eq!(message, "A picture of 16384 by 16385 pixels is too large.");
        assert!(Pixmap::new(u32::MAX, u32::MAX).is_err());
        let empty = Pixmap::new(0, 7).unwrap();
        assert_eq!((empty.bytes().len(), empty.pixel(0, 0)), (0, None));
    }
}
