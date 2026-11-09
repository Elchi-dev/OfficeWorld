//! Unpacking deflate.

use std::error::Error;
use std::fmt;

use crate::codes::{DISTANCES, END, LENGTHS, fixed_lengths};

/// Why compressed data could not be unpacked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InflateError {
    /// The data ends before it is finished.
    Truncated,
    /// The data is not what the format allows: it is damaged, or it is
    /// something else.
    Invalid,
    /// Unpacked, the data would be longer than the caller allowed.
    TooLong,
}

impl fmt::Display for InflateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Truncated => "The compressed data ends before it is finished.",
            Self::Invalid => "The compressed data is damaged.",
            Self::TooLong => "The compressed data unpacks to more than is allowed.",
        })
    }
}

impl Error for InflateError {}

/// Compressed data read bit by bit, the lowest bit of each byte first.
struct Bits<'a> {
    bytes: &'a [u8],
    /// How many bits have been taken.
    at: usize,
}

impl Bits<'_> {
    /// Takes up to 16 bits as a number. The first bit taken is its lowest.
    fn take(&mut self, count: u8) -> Result<u16, InflateError> {
        let mut number = 0;
        for place in 0..count {
            let byte = self.bytes.get(self.at / 8).ok_or(InflateError::Truncated)?;
            number |= u16::from(byte >> (self.at % 8) & 1) << place;
            self.at += 1;
        }
        Ok(number)
    }
}

/// A set of codes of different lengths, none of which begins like another.
/// The shorter a symbol's code, the more often the symbol is expected.
struct Codes {
    /// How many symbols have a code of each length.
    counts: [u16; 16],
    /// The symbols in the order of their codes.
    symbols: Vec<u16>,
}

impl Codes {
    /// The codes that follow from how long the code of each symbol is, with
    /// 0 for a symbol that has none. Lengths that ask for more codes than
    /// there can be are refused.
    fn new(lengths: &[u8]) -> Result<Self, InflateError> {
        let mut counts = [0_u16; 16];
        for &length in lengths {
            counts[usize::from(length)] += 1;
        }
        counts[0] = 0;
        // Every bit a code is longer doubles the codes there is room for.
        let mut room = 1_u32;
        for &count in &counts[1..] {
            let left = (room * 2).checked_sub(u32::from(count));
            room = left.ok_or(InflateError::Invalid)?;
        }
        // Codes are given out to the shortest lengths first and, within a
        // length, in the order of the symbols.
        let coded = (0..).zip(lengths).filter(|(_, length)| **length != 0);
        let mut symbols: Vec<u16> = coded.map(|(symbol, _)| symbol).collect();
        symbols.sort_by_key(|&symbol| lengths[usize::from(symbol)]);
        Ok(Self { counts, symbols })
    }

    /// Takes the code of one symbol.
    fn symbol(&self, bits: &mut Bits) -> Result<u16, InflateError> {
        // The codes of one length are numbers in a row, and the first of
        // them follows from the lengths before. So one more bit is taken
        // until the number read is among the codes of its length.
        let (mut code, mut first, mut before) = (0_u32, 0_u32, 0_usize);
        for &count in &self.counts[1..] {
            code |= u32::from(bits.take(1)?);
            if code < first + u32::from(count) {
                return Ok(self.symbols[before + (code - first) as usize]);
            }
            before += usize::from(count);
            first = (first + u32::from(count)) * 2;
            code *= 2;
        }
        Err(InflateError::Invalid)
    }
}

/// Unpacks a block that was stored as it is.
fn stored(bits: &mut Bits, out: &mut Vec<u8>) -> Result<(), InflateError> {
    // It starts at the next whole byte with its length, written twice: as
    // it is, and with every bit turned over.
    let start = bits.at.div_ceil(8);
    let header = bits.bytes.get(start..start + 4);
    let header = header.ok_or(InflateError::Truncated)?;
    let length = u16::from_le_bytes([header[0], header[1]]);
    if length != !u16::from_le_bytes([header[2], header[3]]) {
        return Err(InflateError::Invalid);
    }
    let end = start + 4 + usize::from(length);
    let block = bits.bytes.get(start + 4..end);
    out.extend_from_slice(block.ok_or(InflateError::Truncated)?);
    bits.at = end * 8;
    Ok(())
}

/// Reads the codes a block brings along: how long the code of each symbol
/// is, written with a third set of codes.
fn own_codes(bits: &mut Bits) -> Result<(Codes, Codes), InflateError> {
    // The order in which the lengths of that third set are written. The
    // ones most often left out come last, so they can be cut off.
    const ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let symbols = usize::from(bits.take(5)?) + 257;
    let distances = usize::from(bits.take(5)?) + 1;
    let written = usize::from(bits.take(4)?) + 4;
    let mut lengths = [0; 19];
    for &place in &ORDER[..written] {
        lengths[place] = bits.take(3)?.to_le_bytes()[0];
    }
    let codes = Codes::new(&lengths)?;
    let mut lengths = Vec::new();
    while lengths.len() < symbols + distances {
        // A length itself, or one of three ways to say "the same again".
        let (length, times) = match codes.symbol(bits)? {
            16 => (lengths.last().copied(), 3 + bits.take(2)?),
            17 => (Some(0), 3 + bits.take(3)?),
            18 => (Some(0), 11 + bits.take(7)?),
            length => (Some(length.to_le_bytes()[0]), 1),
        };
        let length = length.ok_or(InflateError::Invalid)?;
        lengths.extend(std::iter::repeat_n(length, usize::from(times)));
    }
    // The last "the same again" must not run past the last symbol. More
    // symbols than the format knows are let through here: one that means
    // nothing is refused when it is used.
    if lengths.len() != symbols + distances {
        return Err(InflateError::Invalid);
    }
    let (symbols, distances) = lengths.split_at(symbols);
    Ok((Codes::new(symbols)?, Codes::new(distances)?))
}

/// Unpacks a block of bytes and repeats, each written with a code.
fn packed(
    bits: &mut Bits,
    out: &mut Vec<u8>,
    most: usize,
    codes: &(Codes, Codes),
) -> Result<(), InflateError> {
    loop {
        let symbol = codes.0.symbol(bits)?;
        if let Ok(byte) = u8::try_from(symbol) {
            out.push(byte);
        } else if symbol == END {
            return Ok(());
        } else {
            let length = LENGTHS.get(usize::from(symbol - END - 1));
            let &(first, more) = length.ok_or(InflateError::Invalid)?;
            let length = usize::from(first + bits.take(more)?);
            let distance = DISTANCES.get(usize::from(codes.1.symbol(bits)?));
            let &(first, more) = distance.ok_or(InflateError::Invalid)?;
            let distance = usize::from(first + bits.take(more)?);
            // A repeat may reach into itself: it is copied byte by byte.
            let from = out.len().checked_sub(distance);
            let from = from.ok_or(InflateError::Invalid)?;
            for at in from..from + length {
                out.push(out[at]);
            }
        }
        if out.len() > most {
            return Err(InflateError::TooLong);
        }
    }
}

/// Unpacks deflate data, and says how many bytes of it the stream took.
pub(crate) fn stream(data: &[u8], most: usize) -> Result<(Vec<u8>, usize), InflateError> {
    let mut bits = Bits { bytes: data, at: 0 };
    let mut out = Vec::new();
    loop {
        let last = bits.take(1)? == 1;
        match bits.take(2)? {
            0 => stored(&mut bits, &mut out)?,
            1 => {
                let (symbols, distances) = fixed_lengths();
                let codes = (Codes::new(&symbols)?, Codes::new(&distances)?);
                packed(&mut bits, &mut out, most, &codes)?;
            }
            2 => {
                let codes = own_codes(&mut bits)?;
                packed(&mut bits, &mut out, most, &codes)?;
            }
            _ => return Err(InflateError::Invalid),
        }
        if out.len() > most {
            return Err(InflateError::TooLong);
        }
        if last {
            return Ok((out, bits.at.div_ceil(8)));
        }
    }
}

/// Unpacks data that was compressed with deflate, the compression of ZIP,
/// PNG and PDF.
///
/// Compressed data can unpack to a thousand times its size, and a file can
/// be made to. `most` is the longest result the caller is prepared to
/// take: the size a ZIP archive or a picture's header announces.
///
/// # Errors
///
/// Refuses data that ends early, data that is not deflate, and data that
/// unpacks to more than `most` bytes.
pub fn inflate(data: &[u8], most: usize) -> Result<Vec<u8>, InflateError> {
    stream(data, most).map(|(out, _)| out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(hex: &str) -> Vec<u8> {
        let pairs = (0..hex.len()).step_by(2);
        pairs
            .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
            .collect()
    }

    /// "0 squared is 0. 1 squared is 1. " and so on up to 59: its first 40
    /// bytes as zlib 1.3 stores them, its first 200 as zlib packs them
    /// with the fixed codes, and all of it packed with codes of its own.
    const STORED: &str = "012800d7ff30207371756172656420697320302e2031207371756172656420697320312e2032\
        20737175617265";
    const FIXED: &str = "3350282e2c4d2c4a4d51c82c5630d0533044e61bea291821f34df4148c91f9967a0a2628eacd\
        f4144c91058c4cf514cc90058c812acc518c049a61812c6006b4c41259c002e80a4303146b0c\
        400e45752900";
    const OWN: &str = "5d934b6e05210c04afc20946f807c37122258b2c93a7dc3ff4aca8b7c4b2eceaa6dddbebe7ef\
        e3f7ebb37dbf5abf9a9d6fbb9a9fefbc5a9cef75b544ffb85a9d05afab8db310bb6362e49e71\
        9f85b197acb3706f0aeb58d3054a525717602df7240bf2ed6d46e4b5898cd0a2b6411dea02b8\
        df9a05f4706d5cd42b0f419fa277d067aa8b56df7b9683be7c6f74d0d7dc5c0efa217a07fd78\
        ba403f9f59a09fcf465a2fae00fd127d807e4963d0fb2e2b82e677391674df646c14bf520a62\
        b0a66f0a48b0d06fc6fdf6e7da0b1156ca463241433292111a4a5a52c7541c933a6e6536df52\
        a4602763d4a5239923d3892483e45df3a0c3a36b2f7478f6cd579db9948e820e1f4f1f74f8d4\
        bc0a06587b0b3a7c3d270c1df1e8289eb1496fcdb7f46b1e2f22e45ff124523eff03";

    #[test]
    fn data_packed_by_zlib_is_unpacked() {
        let add = |text: String, n: u32| text + &format!("{n} squared is {}. ", n * n);
        let squares = (0..60).fold(String::new(), add);
        for (packed, length) in [(STORED, 40), (FIXED, 200), (OWN, 1144)] {
            let (packed, text) = (bytes(packed), &squares.as_bytes()[..length]);
            assert_eq!(inflate(&packed, length).as_deref(), Ok(text));
            // One byte less than it needs is not enough room, and one byte
            // less than it has is not all of it.
            assert_eq!(inflate(&packed, length - 1), Err(InflateError::TooLong));
            let cut = inflate(&packed[..packed.len() - 1], length);
            assert_eq!(cut, Err(InflateError::Truncated));
        }
    }

    #[test]
    fn data_that_breaks_the_format_is_refused() {
        // A kind of block there is none of. A stored block whose two
        // lengths disagree. A repeat that reaches back before the start.
        // And codes of its own with four codes one bit long, where there
        // is room for two.
        let mut disagreeing = bytes(STORED);
        disagreeing[3] ^= 1;
        let crowded = vec![0x05, 0x00, 0x92, 0x04];
        for broken in [vec![0x07], disagreeing, vec![0x03, 0x02], crowded] {
            assert_eq!(
                inflate(&broken, 100),
                Err(InflateError::Invalid),
                "{broken:?}"
            );
        }
        let said = InflateError::TooLong.to_string();
        assert_eq!(said, "The compressed data unpacks to more than is allowed.");
    }
}
