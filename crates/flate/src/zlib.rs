//! The zlib wrapping of deflate: two bytes in front and a checksum behind.

use crate::checksum::adler32;
use crate::deflate::deflate;
use crate::inflate::{InflateError, stream};

/// Compresses data into a zlib stream, as a PNG file holds its pixels and a
/// PDF file its contents.
pub fn to_zlib(data: &[u8]) -> Vec<u8> {
    // The two bytes say: deflate, reaching back up to 32,768 bytes.
    let parts = [
        &[0x78, 0x9C],
        &deflate(data)[..],
        &adler32(data).to_be_bytes(),
    ];
    parts.concat()
}

/// Unpacks a zlib stream into at most `most` bytes.
///
/// # Errors
///
/// Refuses data that ends early, data that is not a zlib stream or whose
/// checksum does not fit, and data that unpacks to more than `most` bytes.
pub fn from_zlib(data: &[u8], most: usize) -> Result<Vec<u8>, InflateError> {
    let header = data.get(..2).ok_or(InflateError::Truncated)?;
    // The method is deflate, no dictionary is asked for, and the two bytes
    // read as one number divide by 31.
    let deflated = header[0] & 0x0F == 8 && header[1] & 0x20 == 0;
    if !deflated || u16::from_be_bytes([header[0], header[1]]) % 31 != 0 {
        return Err(InflateError::Invalid);
    }
    let (out, taken) = stream(&data[2..], most)?;
    let sum = data.get(2 + taken..6 + taken);
    if sum.ok_or(InflateError::Truncated)? != adler32(&out).to_be_bytes() {
        return Err(InflateError::Invalid);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zlib_stream_is_checked_on_its_way_in() {
        // Nothing, as zlib itself wraps it.
        let nothing = [0x78, 0x9C, 0x03, 0x00, 0x00, 0x00, 0x00, 0x01];
        assert_eq!(to_zlib(b""), nothing);
        let (text, packed) = (&b"an office suite"[..], to_zlib(b"an office suite"));
        assert_eq!(from_zlib(&packed, 15).as_deref(), Ok(text));
        // The same text as zlib wraps it when it is told not to compress:
        // the checksum is found behind a block of any kind.
        let stored = "7801010f00f0ff616e206f66666963652073756974652bdb05a6";
        let byte = |at: usize| u8::from_str_radix(&stored[at..at + 2], 16).unwrap();
        let stored: Vec<u8> = (0..stored.len()).step_by(2).map(byte).collect();
        assert_eq!(from_zlib(&stored, 15).as_deref(), Ok(text));
        // Another method, a dictionary asked for, two first bytes that do
        // not go together, and a checksum that does not fit what came out.
        let end = packed.len() - 2;
        for (at, bytes) in [
            (0, [0x79, 0x18]),
            (0, [0x78, 0x20]),
            (0, [0x78, 0x9D]),
            (end, [0, 0]),
        ] {
            let mut broken = packed.clone();
            broken[at..at + 2].copy_from_slice(&bytes);
            assert_eq!(
                from_zlib(&broken, 15),
                Err(InflateError::Invalid),
                "{bytes:?}"
            );
        }
        for length in [1, packed.len() - 1] {
            assert_eq!(
                from_zlib(&packed[..length], 15),
                Err(InflateError::Truncated)
            );
        }
    }
}
