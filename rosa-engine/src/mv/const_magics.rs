pub const ROOK_PREMASK: [u64; 64] = rook_premask(false);
pub const ROOK_PREMASK_TRUNC: [u64; 64] = rook_premask(true);
pub const BISHOP_PREMASK: [u64; 64] = bishop_premask(false);
pub const BISHOP_PREMASK_TRUNC: [u64; 64] = bishop_premask(true);

pub const ROOK_LOOKUP: [&[u64]; 64] = rook_lookup();
pub const BISHOP_LOOKUP: [&[u64]; 64] = bishop_lookup();

const fn rook_premask(trunc: bool) -> [u64; 64] {
    [0; 64]
}

const fn bishop_premask(trunc: bool) -> [u64; 64] {
    [0; 64]
}

const fn rook_lookup() -> [&'static [u64]; 64] {}
const fn bishop_lookup() -> [&'static [u64]; 64] {}
