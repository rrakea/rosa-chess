use crate::mv::constants::*;
use rosa_lib::pos::Pos;

const OUTSIDE_RANK_MASK: u64 =
    0b1111_1111_1000_0001_1000_0001_1000_0001_1000_0001_1000_0001_1000_0001_1111_1111;

pub fn queen_mask(sq: u8, p: &Pos, cap: bool) -> u64 {
    rook_mask(sq, p, cap) | bishop_mask(sq, p, cap)
}

pub fn rook_mask(sq: u8, p: &Pos, cap: bool) -> u64 {
    let sq = sq as usize;
    let premask = unsafe { ROOK_PREMASKS_TRUNC[sq] };
    let magic = ROOK_MAGIC[sq];
    let shift = ROOK_SHIFT[sq];
    let blocker = premask & p.full().val();
    let index = magic_index(magic, shift, blocker);
    let res = unsafe { ROOK_LOOKUP[sq][index] };
    // Since our blockers dont extend to the outside squares
    // -> Since it doesnt matter if there is a piece there
    // However it does matter for captures
    if cap {
        res & (blocker | OUTSIDE_RANK_MASK)
    } else {
        res & !blocker
    }
}

pub fn bishop_mask(sq: u8, p: &Pos, cap: bool) -> u64 {
    let sq = sq as usize;
    let premask = unsafe { BISHOP_PREMASKS_TRUNC[sq] };
    let magic = BISHOP_MAGIC[sq];
    let shift = BISHOP_SHIFT[sq];
    let blocker = premask & p.full().val();
    let index = magic_index(magic, shift, blocker);
    let res = unsafe { BISHOP_LOOKUP[sq][index] };
    if cap {
        res & (blocker | OUTSIDE_RANK_MASK)
    } else {
        res & !blocker
    }
}

pub fn magic_index(magic: u64, shift: u8, blocker: u64) -> usize {
    (u64::wrapping_mul(magic, blocker) >> (64 - shift)) as usize
}
