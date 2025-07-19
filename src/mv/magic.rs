use crate::mv::constants::*;
use crate::pos::Pos;

pub fn queen_mask(sq: u8, p: &Pos) -> u64 {
    rook_mask(sq, p) | bishop_mask(sq, p)
}

pub fn rook_mask(sq: u8, p: &Pos) -> u64 {
    let sq = sq as usize;
    let premask = unsafe { ROOK_PREMASKS[sq] };
    let magic = ROOK_MAGIC[sq];
    let shift = ROOK_SHIFT[sq];
    let blocker = premask & p.full.val();
    let index = magic_index(magic, shift, blocker);
    unsafe { ROOK_LOOKUP[sq][index] }
}

pub fn bishop_mask(sq: u8, p: &Pos) -> u64 {
    let premask = unsafe { BISHOP_PREMASKS[sq as usize] };
    let magic = BISHOP_MAGIC[sq as usize];
    let shift = BISHOP_SHIFT[sq as usize];
    let blocker = premask & p.full.val();
    let index = magic_index(magic, shift, blocker);
    unsafe { BISHOP_LOOKUP[sq as usize][index] }
}

pub fn magic_index(magic: u64, shift: u8, blocker: u64) -> usize {
    ((magic * blocker) >> (64 - shift)) as usize
}
