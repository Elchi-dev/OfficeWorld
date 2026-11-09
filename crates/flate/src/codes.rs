//! The codes deflate writes with, as its specification, RFC 1951, sets
//! them.

/// A repeat of earlier data is written as how long it is and how far back
/// it reaches. Each of the two is sent as a code that stands for a range
/// of values, followed by enough bits to say which value of the range is
/// meant. For each code of a length: the first value of its range, and how
/// many bits follow.
#[rustfmt::skip]
pub(crate) const LENGTHS: [(u16, u8); 29] = [
    (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0), (10, 0),
    (11, 1), (13, 1), (15, 1), (17, 1), (19, 2), (23, 2), (27, 2), (31, 2),
    (35, 3), (43, 3), (51, 3), (59, 3), (67, 4), (83, 4), (99, 4), (115, 4),
    (131, 5), (163, 5), (195, 5), (227, 5), (258, 0),
];

/// The same for each code of a distance.
#[rustfmt::skip]
pub(crate) const DISTANCES: [(u16, u8); 30] = [
    (1, 0), (2, 0), (3, 0), (4, 0), (5, 1), (7, 1), (9, 2), (13, 2),
    (17, 3), (25, 3), (33, 4), (49, 4), (65, 5), (97, 5), (129, 6), (193, 6),
    (257, 7), (385, 7), (513, 8), (769, 8), (1025, 9), (1537, 9),
    (2049, 10), (3073, 10), (4097, 11), (6145, 11), (8193, 12), (12289, 12),
    (16385, 13), (24577, 13),
];

/// The symbol that ends a block. The symbols below it are the bytes
/// themselves, and the ones above it the codes for lengths.
pub(crate) const END: u16 = 256;

/// How many bits the code of each symbol has in a block that uses the
/// fixed codes: of the 288 symbols for bytes, the end and lengths, and of
/// the 30 for distances.
pub(crate) fn fixed_lengths() -> ([u8; 288], [u8; 30]) {
    let mut symbols = [8; 288];
    symbols[144..256].fill(9);
    symbols[256..280].fill(7);
    (symbols, [5; 30])
}
