//! Compression for `OfficeWorld`.
//!
//! Deflate is the compression inside the files an office suite lives on:
//! every .docx, .xlsx and .pptx is a ZIP archive of it, a PNG picture holds
//! its pixels in it, and so does most of a PDF. This crate packs and
//! unpacks it, bare and in its zlib wrapping, and works out the two
//! checksums that travel with it.

mod checksum;

pub use checksum::{Crc32, adler32};
