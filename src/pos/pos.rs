use crate::board::Board;
// Iternal representation of the current position.
// Using a hybrid approach of both bitboards and square centric

// w/b for color & p/n/b/r/q/k for the pieces using the common abbreviations
pub const WPAWN: i8 = 1;
pub const WBISHOP: i8 = 2;
pub const WKNIGHT: i8 = 3;
pub const WROOK: i8 = 4;
pub const WQUEEN: i8 = 5;
pub const WKING: i8 = 6;

pub const BPAWN: i8 = -1;
pub const BBISHOP: i8 = -2;
pub const BKNIGHT: i8 = -3;
pub const BROOK: i8 = -4;
pub const BQUEEN: i8 = -5;
pub const BKING: i8 = -6;

#[derive(Clone, Debug)]
pub struct Pos {
    // Bitboards
    pub wp: Board,
    pub wn: Board,
    pub wb: Board,
    pub wr: Board,
    pub wq: Board,
    pub wk: Board,

    pub bp: Board,
    pub bn: Board,
    pub bb: Board,
    pub br: Board,
    pub bq: Board,
    pub bk: Board,

    // Full board bitboard
    pub full: Board,

    // Square centric representation
    // Using the consts defined
    // (Negativ for black pieces)
    pub sq: [i8; 64],

    // Extra Data
    // Encoded like this: castling:  b queen, b king, w queen, b king; en_passant file;
    pub data: u8,
    // Active player (1 == w; -1 == b)
    pub active: i8,
    // Zobrist Hashing Key of the current position
    pub key: u64,
}

impl Pos {
    pub fn is_en_passant(&self) -> bool {
        (self.data & 0b0000_1000) != 0
    }

    pub fn en_passant_file(&self) -> u8 {
        self.data & 0b0000_0111
    }

    // -> kingside, queenside
    pub fn castling(&self, color: i8) -> (bool, bool) {
        if color == 1 {
            (self.data & 0b0001_0000 > 0, self.data & 0b0010_0000 > 0)
        } else {
            (self.data & 0b0100_0000 > 0, self.data & 0b1000_0000 > 0)
        }
    }
}

pub fn start_pos() -> Pos {}

pub fn pos_from_fen() -> Pos {}

pub fn gen_data(ep_rank: u8, w_castle: (bool, bool), b_castle: (bool, bool)) -> u8 {}
