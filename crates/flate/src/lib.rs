//! Compression for `OfficeWorld`.
//!
//! Deflate is the compression inside the files an office suite lives on:
//! every .docx, .xlsx and .pptx is a ZIP archive of it, a PNG picture holds
//! its pixels in it, and so does most of a PDF. This crate packs and
//! unpacks it, bare and in its zlib wrapping, and works out the two
//! checksums that travel with it.
//!
//! ```
//! use ow_flate::{from_zlib, to_zlib};
//!
//! let text = "to be or not to be, that is the question".repeat(20);
//! let packed = to_zlib(text.as_bytes());
//! assert!(packed.len() < text.len() / 10);
//!
//! // Data from a file says how long it will be. Whoever unpacks it names
//! // the most they will take, and a file that lies is refused.
//! assert_eq!(from_zlib(&packed, 800).as_deref(), Ok(text.as_bytes()));
//! assert!(from_zlib(&packed, 799).is_err());
//! ```

mod checksum;
mod codes;
mod deflate;
mod inflate;
mod zlib;

pub use checksum::{Crc32, adler32};
pub use deflate::deflate;
pub use inflate::{InflateError, inflate};
pub use zlib::{from_zlib, to_zlib};
