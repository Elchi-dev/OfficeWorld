//! Paths as SVG path data, written and read.

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use ow_geometry::{Point, Vector};

use crate::path::{Element, Path};

impl fmt::Display for Path {
    /// Writes the path as SVG path data, such as `M1 1 L5 1.5 Z`. Every
    /// command is written out with the points themselves, none relative to
    /// the one before.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, element) in self.elements().iter().enumerate() {
            if index > 0 {
                f.write_str(" ")?;
            }
            match element {
                Element::MoveTo(to) => write!(f, "M{} {}", to.x, to.y),
                Element::LineTo(to) => write!(f, "L{} {}", to.x, to.y),
                Element::QuadTo(control, to) => {
                    write!(f, "Q{} {} {} {}", control.x, control.y, to.x, to.y)
                }
                Element::CurveTo(first, second, to) => {
                    let (x1, y1, x2, y2) = (first.x, first.y, second.x, second.y);
                    write!(f, "C{x1} {y1} {x2} {y2} {} {}", to.x, to.y)
                }
                Element::Close => f.write_str("Z"),
            }?;
        }
        Ok(())
    }
}

/// A text that could not be read as SVG path data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPathData {
    /// How many bytes into the text the part that could not be read begins.
    pub position: usize,
}

impl fmt::Display for InvalidPathData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let position = self.position;
        write!(f, "The path data cannot be read from byte {position} on.")
    }
}

impl Error for InvalidPathData {}

/// Path data being read, and how far the reading has come.
struct Reader<'a> {
    data: &'a str,
    position: usize,
}

impl Reader<'_> {
    fn fail<T>(&self) -> Result<T, InvalidPathData> {
        let position = self.position;
        Err(InvalidPathData { position })
    }

    /// The next byte that is not white space or a comma, without taking it.
    fn peek(&mut self) -> Option<u8> {
        let bytes = self.data.as_bytes();
        let apart = |byte: &&u8| byte.is_ascii_whitespace() || **byte == b',';
        self.position += bytes[self.position..].iter().take_while(apart).count();
        bytes.get(self.position).copied()
    }

    /// Takes a number. One number may follow another without anything in
    /// between where a sign or a second point shows that a new one begins,
    /// as in `1-2` and `.5.5`.
    fn number(&mut self) -> Result<f64, InvalidPathData> {
        self.peek();
        let bytes = &self.data.as_bytes()[self.position..];
        let signed = |at: usize| usize::from(matches!(bytes.get(at), Some(b'+' | b'-')));
        let digits = |from: usize| {
            let digits = bytes[from..]
                .iter()
                .take_while(|byte| byte.is_ascii_digit());
            from + digits.count()
        };
        let mut end = digits(signed(0));
        if bytes.get(end) == Some(&b'.') {
            end = digits(end + 1);
        }
        if matches!(bytes.get(end), Some(b'e' | b'E')) {
            end = digits(end + 1 + signed(end + 1));
        }
        // A number too large to hold is refused with the ones that are not
        // numbers at all: nothing drawn from a file is to be infinite.
        let text = &self.data[self.position..self.position + end];
        match text.parse::<f64>() {
            Ok(number) if number.is_finite() => {
                self.position += end;
                Ok(number)
            }
            _ => self.fail(),
        }
    }

    /// Takes a point, counted from an origin.
    fn point(&mut self, origin: Vector) -> Result<Point, InvalidPathData> {
        Ok(Point::new(self.number()?, self.number()?) + origin)
    }

    /// Takes one of the two choices of an arc: a single `0` or `1`, which
    /// may be followed by the next number without anything in between.
    fn flag(&mut self) -> Result<bool, InvalidPathData> {
        let flag = match self.peek() {
            Some(b'0') => false,
            Some(b'1') => true,
            _ => return self.fail(),
        };
        self.position += 1;
        Ok(flag)
    }
}

impl FromStr for Path {
    type Err = InvalidPathData;

    /// Reads SVG path data: every command of it, with points given outright
    /// or relative to the one before, such as `M1 1 l4 .5 z`.
    ///
    /// Arcs are read into the cubic curves that draw them. Data that does
    /// not begin with a move is drawn the way [`Path::line_to`] draws on an
    /// empty path. Data with a mistake in it is refused as a whole.
    fn from_str(data: &str) -> Result<Self, Self::Err> {
        let mut reader = Reader { data, position: 0 };
        let mut path = Self::new();
        let mut command = b' ';
        // The control point of the curve before, for a smooth curve of the
        // same kind to mirror: of a cubic one, and of a quadratic one.
        let (mut cubic, mut quad) = (None, None);
        // Every command takes at least a byte, so there are no more of them
        // than bytes, and the reading ends whatever the data holds.
        for _ in 0..data.len() {
            let Some(next) = reader.peek() else { break };
            // Numbers without a command in front of them repeat the command
            // before.
            let named = next.is_ascii_alphabetic();
            if named {
                command = next;
            }
            let here = path.current().unwrap_or(Point::ORIGIN);
            let relative = command.is_ascii_lowercase();
            let origin = if relative {
                here.to_vector()
            } else {
                Vector::ZERO
            };
            let mirrored = |control: Option<Point>| control.map_or(here, |it| here + (here - it));
            let (cubic_before, quad_before) = (cubic.take(), quad.take());
            let kind = command.to_ascii_uppercase();
            let position = reader.position;
            reader.position += usize::from(named);
            match kind {
                b'M' => {
                    path.move_to(reader.point(origin)?);
                    // Points that follow a move without a command of their
                    // own are lines.
                    command = if relative { b'l' } else { b'L' };
                }
                b'L' => {
                    path.line_to(reader.point(origin)?);
                }
                b'H' => {
                    path.line_to((reader.number()? + origin.x, here.y));
                }
                b'V' => {
                    path.line_to((here.x, reader.number()? + origin.y));
                }
                b'Q' | b'T' => {
                    let control = match kind {
                        b'Q' => reader.point(origin)?,
                        _ => mirrored(quad_before),
                    };
                    path.quad_to(control, reader.point(origin)?);
                    quad = Some(control);
                }
                b'C' | b'S' => {
                    let first = match kind {
                        b'C' => reader.point(origin)?,
                        _ => mirrored(cubic_before),
                    };
                    let second = reader.point(origin)?;
                    path.curve_to(first, second, reader.point(origin)?);
                    cubic = Some(second);
                }
                b'A' => {
                    let radii = (reader.number()?, reader.number()?);
                    let turn = reader.number()?.to_radians();
                    let (large, clockwise) = (reader.flag()?, reader.flag()?);
                    let to = reader.point(origin)?;
                    path.arc_to(radii.0, radii.1, turn, large, clockwise, to);
                }
                // A close takes no numbers, so there is nothing to repeat.
                b'Z' if named => {
                    path.close();
                }
                _ => return Err(InvalidPathData { position }),
            }
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rewritten(data: &str) -> String {
        data.parse::<Path>().unwrap().to_string()
    }

    #[test]
    fn every_command_is_read_outright_and_relative() {
        let lines = rewritten("M10 10 l5 0 H20 h5 V20 v5 z m1 1 2 2 M5 5 6 6");
        let expected = "M10 10 L15 10 L20 10 L25 10 L25 20 L25 25 Z M11 11 L13 13 M5 5 L6 6";
        assert_eq!(lines, expected);
        // A smooth curve mirrors the last control point of the curve before
        // it, where that was a curve of its own kind.
        let quads = rewritten("M10 20 Q30 30 40 20 t10 0 T60 20 q5 5 10 0");
        assert_eq!(
            quads,
            "M10 20 Q30 30 40 20 Q50 10 50 20 Q50 30 60 20 Q65 25 70 20"
        );
        let cubics =
            rewritten("M70 20 C80 30 90 30 90 20 s10 -10 10 0 S110 30 120 20 c1 1 2 2 3 0");
        let expected = "M70 20 C80 30 90 30 90 20 C90 10 100 10 100 20 \
                        C100 30 110 30 120 20 C121 21 122 22 123 20";
        assert_eq!(cubics, expected);
        let mixed = rewritten("M0 0 Q5 5 10 0 S20 10 30 0 T40 0");
        assert_eq!(mixed, "M0 0 Q5 5 10 0 C10 0 20 10 30 0 Q30 0 40 0");
    }

    #[test]
    fn numbers_and_flags_need_nothing_between_them() {
        assert_eq!(
            rewritten("M-.5.5e1,1E-1+2\n3.25e+1-4."),
            "M-0.5 5 L0.1 2 L32.5 -4"
        );
        assert_eq!(rewritten(" , M1,1Z"), "M1 1 Z");
        // An arc is given its angle in degrees and its two choices as single
        // digits, here run together with the point that follows.
        let quarter = 90.0_f64.to_radians();
        let mut arc = Path::new();
        arc.move_to((1.0, 2.0));
        arc.arc_to(5.0, 3.0, quarter, false, true, (7.0, 2.0));
        arc.arc_to(4.0, 4.0, 0.0, true, false, (9.0, 9.0));
        assert_eq!("M1 2a5 3 90 016 0A4 4 0 1 0 9 9".parse(), Ok(arc));
    }

    #[test]
    fn data_with_a_mistake_is_refused_where_the_mistake_is() {
        // In each of these, ^ marks the byte at which the reading stops: no
        // command, a number missing, a command there is none of, a close
        // repeated, half a number, a number too large, and a choice that is
        // neither 0 nor 1.
        let mistakes =
            "^1 2|M1^|M1 1 ^X5|M1 1 Z ^2 2|M1 ^. 2|M1 ^1e|M^1e999 0|M0 0 A1 1 0 ^2 0 5 5";
        for marked in mistakes.split('|') {
            let (data, position) = (marked.replace('^', ""), marked.find('^').unwrap());
            assert_eq!(
                data.parse::<Path>(),
                Err(InvalidPathData { position }),
                "{marked}"
            );
        }
        let message = InvalidPathData { position: 7 }.to_string();
        assert_eq!(message, "The path data cannot be read from byte 7 on.");
    }
}
