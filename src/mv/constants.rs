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

// The magics shift are all equivalent to the "best magics so far page" on the chess programming wiki
// -> The magic gen should be correct :)))
pub const ROOK_MAGIC: [u64; 64] = [
    0x4080009021804000,
    0x00400240d0082000,
    0xa480200080100008,
    0x8080041800811002,
    0x0600054810200200,
    0x8100240012080100,
    0x1480020001002080,
    0x4080004421000180,
    0x9080802280004000,
    0x1500808040002000,
    0x5881002000410114,
    0x0000800800805000,
    0x0004800800140080,
    0x0000800200440080,
    0x0801000500040e00,
    0x810200004101840a,
    0xa900208000804002,
    0x1148c04004201000,
    0x4000120020820040,
    0xa011010010004820,
    0x200c00800800c482,
    0x0c4c008002008004,
    0x001054001008110a,
    0x0000420010408114,
    0x00e0400980012881,
    0x0820400040201000,
    0x0240124100200103,
    0x0024100180280081,
    0x0808000404002040,
    0x8104004040220100,
    0x0000060c00051018,
    0x0400004200011084,
    0x0401604003800880,
    0x0210004000402000,
    0x0108448202001020,
    0x0200090021001001,
    0x0008005401801880,
    0x0210040080800200,
    0x0088100824000902,
    0x0000005406000081,
    0xc204208040008002,
    0x0060004130004008,
    0x00010040a0090010,
    0x002020100101000c,
    0x0802000418120020,
    0x1402000804020090,
    0x0d000150120c0048,
    0x0008210040860004,
    0x0001008046043200,
    0x0410002004400840,
    0x0410001020008080,
    0x4000201001cd0100,
    0x0015810800040180,
    0x1002002450880200,
    0x000200084c014200,
    0x000040410400b600,
    0x3000410130288001,
    0x8002004020108102,
    0xd014402000090031,
    0x0208881000046101,
    0x0002002028241062,
    0x000300080400420b,
    0x0001123110280084,
    0x40002a422100840a,
];

pub const ROOK_SHIFT: [u8; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];

pub const BISHOP_MAGIC: [u64; 64] = [
    0x8012200404004440,
    0x1034900401102014,
    0x0834010202001100,
    0x0304104200400000,
    0x0681104000002102,
    0x1082061104000000,
    0x0204008434200000,
    0x00045404880c3001,
    0x10040e10100a0480,
    0x0100048808010020,
    0x005548180700a8a1,
    0x41400404008c0ca0,
    0x5000040504000400,
    0x0020060104200001,
    0x500008841002120c,
    0x0800644044042000,
    0x0140002004410200,
    0x21202016644c0080,
    0x0081000806012200,
    0x1012008405220204,
    0x0001008820081420,
    0x018541020101a000,
    0x0004002211041204,
    0x0800402200460890,
    0x0a022200100c1000,
    0x0002022308b00400,
    0x8004013010004080,
    0x22c01040a4004080,
    0x0003001011004010,
    0x02080280080a0104,
    0x1405090804480880,
    0x2404420000410421,
    0x8008204450105400,
    0x0088080200040400,
    0x0801280300080a00,
    0x1002208060080200,
    0x004c080201002008,
    0x0501020200008804,
    0x107004008840ca40,
    0xc000820a08248080,
    0x1001246004102000,
    0x4010880412041002,
    0x0110205048003000,
    0x014006c010400a00,
    0x4001012011044200,
    0x0020141000800048,
    0x80100218004c1900,
    0x6402080860840108,
    0x000406080c068000,
    0x0422240928080024,
    0x4000084200900081,
    0x006400002a080400,
    0x0200044022860101,
    0x4200082008088241,
    0xa1a0200102008009,
    0x0004010424048092,
    0x0010104202202002,
    0x00400d0041502808,
    0x3010020200840430,
    0x0040040402050400,
    0x9c000004104a0200,
    0xb0000508104bc200,
    0x0010042012020208,
    0x00081184048c0100,
];
pub const BISHOP_SHIFT: [u8; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];
