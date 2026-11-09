//! Packing deflate.

use crate::codes::{DISTANCES, END, LENGTHS};

/// How far back a repeat may reach.
const WINDOW: usize = 1 << 15;

/// The longest and the shortest repeat that can be written.
const LONGEST: usize = 258;
const SHORTEST: usize = 3;

/// How many earlier places that begin the same way are tried for the
/// longest repeat. More finds longer ones and takes longer.
const TRIES: usize = 128;

/// Compressed data written bit by bit, the lowest bit of each byte first.
#[derive(Default)]
struct Bits {
    bytes: Vec<u8>,
    /// The bits that do not fill a byte yet.
    waiting: u64,
    count: u32,
}

impl Bits {
    /// Writes the lowest bits of a number, the lowest first.
    fn put(&mut self, number: u32, count: u32) {
        self.waiting |= u64::from(number) << self.count;
        self.count += count;
        // The bytes that are full go out, and what is left waits on.
        let full = (self.count / 8) as usize;
        self.bytes
            .extend_from_slice(&self.waiting.to_le_bytes()[..full]);
        self.waiting >>= 8 * full;
        self.count %= 8;
    }

    /// Writes a code, which unlike a number goes out highest bit first.
    fn code(&mut self, code: u32, length: u32) {
        self.put(code.reverse_bits() >> (32 - length), length);
    }

    /// Writes a byte, the end of the block or the code of a length, with
    /// the fixed codes.
    fn symbol(&mut self, symbol: u16) {
        // From each of these symbols on, the codes count up from a first
        // code of some length.
        const RANGES: [(u16, u32, u32); 4] =
            [(0, 0x30, 8), (144, 0x190, 9), (256, 0, 7), (280, 0xC0, 8)];
        let range = RANGES.partition_point(|&(first, ..)| first <= symbol) - 1;
        let (first, code, length) = RANGES[range];
        self.code(code + u32::from(symbol - first), length);
    }

    /// Writes a repeat: so many bytes, the same as so many bytes back.
    fn repeat(&mut self, length: usize, distance: usize) {
        // The code whose range holds a value, and the value's place in it.
        let range = |ranges: &[(u16, u8)], value: usize| {
            let code = ranges.partition_point(|&(first, _)| usize::from(first) <= value) - 1;
            let (first, more) = ranges[code];
            (code, value - usize::from(first), u32::from(more))
        };
        let (code, place, more) = range(&LENGTHS, length);
        self.symbol(END + 1 + u16::try_from(code).unwrap_or(0));
        self.put(u32::try_from(place).unwrap_or(0), more);
        let (code, place, more) = range(&DISTANCES, distance);
        self.code(u32::try_from(code).unwrap_or(0), 5);
        self.put(u32::try_from(place).unwrap_or(0), more);
    }
}

/// The places earlier in the data that begin with the same three bytes.
struct Earlier<'a> {
    data: &'a [u8],
    /// For each beginning, where it was last seen.
    latest: Vec<usize>,
    /// For each place of the last `WINDOW`, where its beginning was seen
    /// before it.
    before: Vec<usize>,
}

impl Earlier<'_> {
    /// A number for the three bytes that begin at a place, or `None` this
    /// close to the end.
    fn beginning(&self, at: usize) -> Option<usize> {
        let three = self.data.get(at..at + SHORTEST)?;
        // Multiplied by a large odd number, the 15 highest bits depend on
        // every bit of the three bytes.
        let number = u32::from_le_bytes([three[0], three[1], three[2], 0]);
        Some((number.wrapping_mul(0x9E37_79B1) >> 17) as usize)
    }

    /// Remembers a place.
    fn note(&mut self, at: usize) {
        if let Some(beginning) = self.beginning(at) {
            self.before[at % WINDOW] = self.latest[beginning];
            self.latest[beginning] = at;
        }
    }

    /// The longest repeat of earlier data that starts at a place: its
    /// length and how far back it reaches. A length of 0 means none.
    fn longest(&self, at: usize) -> (usize, usize) {
        let most = LONGEST.min(self.data.len() - at);
        let mut best = (0, 0);
        let mut earlier = self
            .beginning(at)
            .map_or(usize::MAX, |beginning| self.latest[beginning]);
        for _ in 0..TRIES {
            // No earlier place, or one too far back: so are all before it.
            let Some(back) = at.checked_sub(earlier).filter(|back| *back <= WINDOW) else {
                break;
            };
            let same = |offset: &usize| self.data[earlier + offset] == self.data[at + offset];
            let length = (0..most).take_while(same).count();
            if length > best.0 {
                best = (length, back);
            }
            // Nothing is longer than as long as can be.
            if length == most {
                break;
            }
            earlier = self.before[earlier % WINDOW];
        }
        best
    }
}

/// Compresses data with deflate, the compression of ZIP, PNG and PDF.
///
/// The result is one block that writes its bytes and repeats with the
/// fixed codes of the format. Any program that unpacks deflate reads it.
/// It is not the smallest the format allows: a block that brings its own
/// codes, fitted to the data, is smaller, and is not built yet.
pub fn deflate(data: &[u8]) -> Vec<u8> {
    let mut bits = Bits::default();
    // The last block, and it uses the fixed codes.
    bits.put(0b011, 3);
    let mut earlier = Earlier {
        data,
        latest: vec![usize::MAX; WINDOW],
        before: vec![usize::MAX; WINDOW],
    };
    // How far the data is written.
    let mut written = 0;
    for (at, &byte) in data.iter().enumerate() {
        if at == written {
            let (length, distance) = earlier.longest(at);
            if length < SHORTEST {
                bits.symbol(u16::from(byte));
                written += 1;
            } else {
                bits.repeat(length, distance);
                written += length;
            }
        }
        earlier.note(at);
    }
    bits.symbol(END);
    // The bits that are left get a byte of their own.
    bits.put(0, 7);
    bits.bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inflate::inflate;

    /// Bytes without a pattern.
    fn noise(length: usize) -> Vec<u8> {
        let mut seed = 2_463_534_242_u32;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed.to_le_bytes()[0]
        };
        (0..length).map(|_| next()).collect()
    }

    /// How long data is when packed, after making sure it unpacks to what
    /// it was.
    fn packed(data: &[u8]) -> usize {
        let packed = deflate(data);
        assert_eq!(inflate(&packed, data.len()).as_deref(), Ok(data));
        packed.len()
    }

    #[test]
    fn what_is_packed_unpacks_to_what_it_was() {
        // Nothing is the two bytes zlib makes of it.
        assert_eq!(deflate(b""), [0x03, 0x00]);
        assert_eq!((packed(b""), packed(b"a"), packed(b"abab")), (2, 3, 6));
        // Of two earlier places that are as good, the nearer is cheaper.
        assert_eq!(packed(b"abc1abc2abc3abc4abc5abc6abc7abc8"), 23);
        // A hundred thousand times the same byte: the byte once, then
        // repeats of the greatest length that reach back by one.
        assert_eq!(packed(&vec![b'a'; 100_000]), 634);
        let text = "to be or not to be, that is the question. ".repeat(20);
        assert_eq!(packed(text.as_bytes()), 49);
        // Without a pattern there is nothing to gain, and at nine bits for
        // the upper half of the bytes a sixteenth to lose.
        let noise = noise(70_000);
        assert!(packed(&noise) < 70_000 * 17 / 16 + 8);
    }

    #[test]
    fn a_repeat_reaches_back_as_far_as_the_format_allows_and_no_further() {
        // 300 bytes that came before, from exactly as far back as a repeat
        // can reach and from one byte further.
        let noise = noise(40_000);
        let again = |back: usize| packed(&[&noise[..back], &noise[..300]].concat());
        let alone = |back: usize| packed(&noise[..back]);
        assert!(again(WINDOW) < alone(WINDOW) + 10);
        assert!(again(WINDOW + 1) > alone(WINDOW + 1) + 250);
    }
}
