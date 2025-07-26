use crate::pos;

// Masks when we want to check for specific file/ rank
// e.g. When cheching if a pawn can queen on the next turn
// RANK[0] corresponds to RANK 1 (not like they are displayed here)
pub fn get_mask(piece: i8, sq: u8) -> u64 {
    let sq = sq as usize;
    unsafe {
        match piece {
            pos::BISHOP | pos::BBISHOP => BISHOP_PREMASKS[sq],
            pos::KNIGHT | pos::BKNIGHT => KNIGHT_MASKS[sq],
            pos::ROOK | pos::BROOK => ROOK_PREMASKS[sq],
            pos::KING | pos::BKING => KING_MASKS[sq],
            pos::QUEEN | pos::BQUEEN => ROOK_PREMASKS[sq] | BISHOP_PREMASKS[sq],
            pos::PAWN => WPAWN_MASKS[sq],
            pos::BPAWN => BPAWN_MASKS[sq],
            _ => {

                scream!("get_mask() called with invalid value, piece {}", piece);
            }
        }
    }
}

pub fn get_pawn_mask(active: i8, sq: u8, cap: bool) -> u64 {
    let sq = sq as usize;
    unsafe {
        match (active, cap) {
            (1, false) => WPAWN_MASKS[sq],
            (1, true) => WPAWN_MASKS_CAP[sq],
            (-1, false) => BPAWN_MASKS[sq],
            (-1, true) => BPAWN_MASKS_CAP[sq],
            _ => scream!("Invalid color value: {}, {}", active, cap),
        }
    }
}

pub const RANK_MASKS: [u64; 8] = [
    0x00000000000000FF,
    0x000000000000FF00,
    0x0000000000FF0000,
    0x00000000FF000000,
    0x000000FF00000000,
    0x0000FF0000000000,
    0x00FF000000000000,
    0xFF00000000000000,
];

pub const FILE_MASKS: [u64; 8] = [
    0x0101010101010101,
    0x0202020202020202,
    0x0404040404040404,
    0x0808080808080808,
    0x1010101010101010,
    0x2020202020202020,
    0x4040404040404040,
    0x8080808080808080,
];

<<<<<<< HEAD
pub fn get_mask(piece: i8, sq: u8) -> u64 {
    let sq = sq as usize;
    unsafe {
        match piece {
            pos::BISHOP | pos::BBISHOP => BISHOP_PREMASKS[sq],
            pos::KNIGHT | pos::BKNIGHT => KNIGHT_MASKS[sq],
            pos::ROOK | pos::BROOK => ROOK_PREMASKS[sq],
            pos::KING | pos::BKING => KING_MASKS[sq],
            pos::QUEEN | pos::BQUEEN => ROOK_PREMASKS[sq] | BISHOP_PREMASKS[sq],
            pos::PAWN => WPAWN_MASKS[sq],
            pos::BPAWN => BPAWN_MASKS[sq],
            _ => {
                let error = "get_mask() called with invalid value";
                log::error!("{}, {piece}", error);
                panic!("{}, {piece}", error);
            }
        }
    }
}

pub fn get_pawn_mask(active: i8, sq: u8, cap: bool) -> u64 {
    let sq = sq as usize; unsafe {
        match (active, cap) {
            (1, false) => WPAWN_MASKS[sq],
            (1, true) => WPAWN_MASKS_CAP[sq],
            (-1, false) => BPAWN_MASKS[sq],
            (-1, true) => WPAWN_MASKS_CAP[sq],
            _ => panic!("Invalid color value: {}, {}", active, cap),
        }
    }
}
=======
>>>>>>> e435b6bbcd4c579e653fbc6d78b7a1d9af631c7c

pub const BISHOP_OFFSETS: [i8; 4] = [7, 9, -7, -9];
pub const ROOK_OFFSETS: [i8; 4] = [1, -1, 8, -8];
pub const KING_OFFSETS: [i8; 8] = [1, -1, 8, -8, 7, -7, 9, -9];
pub const KNIGHT_OFFSETS: [i8; 8] = [-10, 6, 15, 17, 10, -6, -15, -17];

// premask = moves on an empty board
pub static mut BISHOP_PREMASKS: [u64; 64] = [0; 64];
pub static mut ROOK_PREMASKS: [u64; 64] = [0; 64];
pub static mut KNIGHT_MASKS: [u64; 64] = [0; 64];
pub static mut KING_MASKS: [u64; 64] = [0; 64];
pub static mut WPAWN_MASKS: [u64; 64] = [0; 64];
pub static mut BPAWN_MASKS: [u64; 64] = [0; 64];
pub static mut WPAWN_MASKS_CAP: [u64; 64] = [0; 64];
pub static mut BPAWN_MASKS_CAP: [u64; 64] = [0; 64];

pub static mut ROOK_PREMASKS_TRUNC: [u64; 64] = [0; 64];
pub static mut BISHOP_PREMASKS_TRUNC: [u64; 64] = [0; 64];

// what we actually index into when we calculate our index with the magics
pub static mut ROOK_LOOKUP: [Vec<u64>; 64] = [const { Vec::new() }; 64];
pub static mut BISHOP_LOOKUP: [Vec<u64>; 64] = [const { Vec::new() }; 64];

// Precalculated
pub const ROOK_MAGIC: [u64; 64] = [
    0x2480002113804000,
    0x0140024410012000,
    0x0900200188410010,
    0x0080100008000482,
    0x0600020008200450,
    0x020002000110880c,
    0x0080010002001380,
    0x0080070000442080,
    0x0002002482024100,
    0x6100c00050052000,
    0x16b0805000842000,
    0x2101801000880080,
    0x0411800400804800,
    0x0009808012000400,
    0x8002000a00010804,
    0x0025000100184282,
    0x0102818002304004,
    0x0440830040042100,
    0x2003410011042000,
    0x00d0008010800800,
    0x000202001008a004,
    0x42210100280c0042,
    0x0000040010013218,
    0x001002000c204189,
    0x700040008004a080,
    0x8601020200408160,
    0x039c814200205200,
    0x0010008080100800,
    0x10028c0280080080,
    0x10040a0080800400,
    0x8400040100020100,
    0x4408608200104401,
    0x00400c8040800c20,
    0x00b0002010400140,
    0x0800802004801000,
    0xc010002804801081,
    0x0018004004040020,
    0x0002011002000498,
    0x001402080c000110,
    0x00401884220000c1,
    0x0094a04000888000,
    0x4204a0005000c004,
    0x0001004560010010,
    0x482800809000800c,
    0x2008008104018008,
    0x142200100c060019,
    0x0823020108240010,
    0x002a010040820004,
    0x10c90b4280002100,
    0x2000802014400080,
    0x0020010010402300,
    0x0008001006800880,
    0x0080800800040180,
    0x8000820004008080,
    0x240008108a010400,
    0x4808050080cc0200,
    0x20402058c0820102,
    0xc400400701108021,
    0x422550200100410d,
    0x0106004020883006,
    0x91020020d0444802,
    0x2109001400120881,
    0x0158808208013004,
    0x010016440092a102,
];

pub const ROOK_SHIFT: [u8; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12
];

pub const BISHOP_MAGIC: [u64; 64] = [0; 64];
pub const BISHOP_SHIFT: [u8; 64] = [0; 64];
