//! Packs or unpacks what comes in on standard input and writes the result
//! to standard output, so that other programs can be tried against this
//! crate. `tools/check_flate.py` does that with zlib.

use std::error::Error;
use std::io::{Read, Write, stdin, stdout};

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = Vec::new();
    stdin().read_to_end(&mut input)?;
    let most = 1 << 30;
    let output = match std::env::args().nth(1).as_deref() {
        Some("deflate") => ow_flate::deflate(&input),
        Some("inflate") => ow_flate::inflate(&input, most)?,
        Some("to-zlib") => ow_flate::to_zlib(&input),
        Some("from-zlib") => ow_flate::from_zlib(&input, most)?,
        _ => return Err("Say deflate, inflate, to-zlib or from-zlib.".into()),
    };
    stdout().write_all(&output)?;
    Ok(())
}
