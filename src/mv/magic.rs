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
    let blocker = premask & p.full.get_val();
    let index = (blocker * magic) >> shift;
    let movemask = unsafe { ROOK_LOOKUP[sq][index as usize] };
    movemask
}

pub fn bishop_mask(sq: u8, p: &Pos) -> u64 {
    let premask = unsafe { BISHOP_PREMASKS[sq as usize] };
    let magic = BISHOP_MAGIC[sq as usize];
    let shift = BISHOP_SHIFT[sq as usize];
    let blocker = premask & p.full.get_val();
    let index = (blocker * magic) >> shift;
    let movemask = unsafe { BISHOP_LOOKUP[sq as usize][index as usize] };
    movemask
}
