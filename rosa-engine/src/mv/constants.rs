use rosa_lib::piece::*;

pub fn get_mask(piece: ClrPiece, sq: u8) -> u64 {
    let sq = sq as usize;
    unsafe {
        match piece {
            ClrPiece::WBishop | ClrPiece::BBishop => BISHOP_PREMASKS[sq],
            ClrPiece::WKnight | ClrPiece::BKnight => KNIGHT_MASKS[sq],
            ClrPiece::WRook | ClrPiece::BRook => ROOK_PREMASKS[sq],
            ClrPiece::WKing | ClrPiece::BKing => KING_MASKS[sq],
            ClrPiece::WQueen | ClrPiece::BQueen => ROOK_PREMASKS[sq] | BISHOP_PREMASKS[sq],
            ClrPiece::WPawn => WPAWN_MASKS[sq],
            ClrPiece::BPawn => BPAWN_MASKS[sq],
        }
    }
}

pub fn get_pawn_mask(clr: Clr, sq: u8, cap: bool) -> u64 {
    let sq = sq as usize;
    unsafe {
        match (clr, cap) {
            (Clr::White, false) => WPAWN_MASKS[sq],
            (Clr::White, true) => WPAWN_MASKS_CAP[sq],
            (Clr::Black, false) => BPAWN_MASKS[sq],
            (Clr::Black, true) => BPAWN_MASKS_CAP[sq],
        }
    }
}

// Masks when we want to check for specific file/ rank
// e.g. When cheching if a pawn can queen on the next turn
// RANK[0] corresponds to RANK 1 (not like they are displayed here)
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
    0x048000804000201a,
    0x0480188020004000,
    0x4100200100501840,
    0x0600060030082140,
    0x0200108882002004,
    0x0600081200041081,
    0x1880090000800200,
    0x02000220c2030084,
    0x10208000e0401082,
    0x8000802000400c84,
    0x8001803001832000,
    0x0011000904100120,
    0x0441800800ac0080,
    0x28d4808014002200,
    0x000a000824010200,
    0x000100010000a042,
    0x4090208000401080,
    0x802000c000403004,
    0x9000410020001100,
    0x2000808030000804,
    0x2403010004080110,
    0x4000808002007400,
    0x0033940010020308,
    0x0840020001018944,
    0x4040012080088240,
    0x001004424000a018,
    0x0000100080200080,
    0x0003a00900300301,
    0x0891000500180050,
    0x8002008080220400,
    0x0042000200018804,
    0x00400402000180c1,
    0x1000400280800528,
    0x0006400080802000,
    0x0041002009001044,
    0x0410000800801182,
    0x0000800400800800,
    0x0200801400800601,
    0x80019021c4000208,
    0x500000c102000384,
    0x0020401463828000,
    0x024040a010044000,
    0x020140b202820021,
    0x0800880010008080,
    0x0000050008010011,
    0x020200b008a20004,
    0x8040420004010100,
    0xc108508b08420024,
    0x01044a2480030b00,
    0x800182a000400080,
    0x0030910660014300,
    0x0940801001280080,
    0x0408020040840040,
    0x9020220080040080,
    0x0062004108040a00,
    0x2000801900004480,
    0x0020402082003502,
    0x0080208213004202,
    0x0401090030406001,
    0x000d001489100021,
    0x0412000450200806,
    0x4409000806c40011,
    0x8000389008053204,
    0x2820010288442402,
];

pub const ROOK_SHIFT: [u8; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];

pub const BISHOP_MAGIC: [u64; 64] = [
    0x0042103000808980,
    0x10200801d0645809,
    0x0104040086040050,
    0x24040c00960002aa,
    0x50040c2010000108,
    0x8006121044000044,
    0x0001455808402010,
    0x0002030108020240,
    0x06500a8888028400,
    0x0000021004052040,
    0x2e0050a404404040,
    0x0100082060400030,
    0x0000511041a50500,
    0x0000110109400040,
    0x1050020086601128,
    0x0000008168080400,
    0x1004c84004044400,
    0x0024002208080900,
    0x0408089000a060a0,
    0x0208220404001000,
    0x20c2100401041004,
    0x0080605410080840,
    0x01010800941002c0,
    0x0026102088410800,
    0x80102a0041083100,
    0x0038600402420202,
    0x2001300088008324,
    0x0600480012820140,
    0x05c9001013004000,
    0x0200410002101a04,
    0x000900408228140c,
    0x0814008902a20101,
    0x0010080800220201,
    0x4300904400284801,
    0x0124002418080040,
    0x0000020080280080,
    0x150c004200040108,
    0x02200800a0004401,
    0x30440a8400008400,
    0x00028c0c40008a10,
    0x88480108088020c0,
    0x2004441008040400,
    0x400100148a041000,
    0x0002004010400200,
    0x1002408092010b00,
    0x0005111003000480,
    0x0020410204800204,
    0x0408010d02040030,
    0x00e40c0304112002,
    0x0a05c24818080005,
    0x00c2402308080048,
    0x0800210084040010,
    0x0000010405040403,
    0x0040080208020000,
    0x0021425012008028,
    0x000c011401020808,
    0x01010149008840c4,
    0x010a114202108200,
    0x0000400201008800,
    0x00000a1800420200,
    0x40800000c0150100,
    0x000a043020839303,
    0x0000102002208220,
    0x0228162c18022b00,
];

pub const BISHOP_SHIFT: [u8; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];
