//! The two checksums that travel with compressed data.

/// What each value of a byte does to a CRC-32, worked out once.
const TABLE: [u32; 256] = {
    let mut table = [0; 256];
    let mut byte = 0_u32;
    while byte < 256 {
        let mut sum = byte;
        let mut bit = 0;
        while bit < 8 {
            // The polynomial of the checksum, low bit first.
            sum = (sum >> 1) ^ (0xEDB8_8320 * (sum & 1));
            bit += 1;
        }
        table[byte as usize] = sum;
        byte += 1;
    }
    table
};

/// A CRC-32 checksum over data that comes in parts: the checksum of PNG,
/// ZIP and gzip.
///
/// ```
/// use ow_flate::Crc32;
///
/// let whole = Crc32::new().with(b"123456789").sum();
/// let in_parts = Crc32::new().with(b"1234").with(b"56789").sum();
/// assert_eq!((whole, in_parts), (0xCBF4_3926, 0xCBF4_3926));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crc32(u32);

impl Crc32 {
    /// The checksum of no data yet.
    pub const fn new() -> Self {
        Self(u32::MAX)
    }

    /// The checksum with more data taken in.
    #[must_use]
    pub fn with(self, bytes: &[u8]) -> Self {
        let step = |sum: u32, byte: &u8| {
            let [low, ..] = sum.to_le_bytes();
            TABLE[usize::from(low ^ byte)] ^ (sum >> 8)
        };
        Self(bytes.iter().fold(self.0, step))
    }

    /// The checksum of everything taken in so far.
    pub const fn sum(self) -> u32 {
        !self.0
    }
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

/// The Adler-32 checksum of data: the checksum at the end of a zlib stream.
pub fn adler32(bytes: &[u8]) -> u32 {
    // Two running sums, the second a sum of the first. They are taken back
    // below the largest prime under 65,536 before they can overflow, which
    // 5,552 bytes at a time cannot make them do.
    let (mut plain, mut summed) = (1_u32, 0_u32);
    for part in bytes.chunks(5552) {
        for &byte in part {
            plain += u32::from(byte);
            summed += plain;
        }
        plain %= 65_521;
        summed %= 65_521;
    }
    (summed << 16) + plain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_checksums_are_the_ones_zlib_gives() {
        assert_eq!(Crc32::new().with(b"123456789").sum(), 0xCBF4_3926);
        assert_eq!((Crc32::default().sum(), adler32(b"")), (0, 1));
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
        // Long enough for the sums to overflow if they were left to grow.
        assert_eq!(adler32(&vec![0xFF; 100_000]), 0x149A_302C);
    }
}
