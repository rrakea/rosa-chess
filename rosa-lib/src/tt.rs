//! # Transposition Table
//! ## Safety
//! The is no read write write - reading might produce smeared data
//! ## Zobrist Hashing

use crate::mv::Mv;
use crate::piece::*;
use crate::pos;

use rand::RngCore;
use std::cell::UnsafeCell;

#[derive(Default)]
pub struct TT {
    table: UnsafeCell<Vec<Option<Entry>>>,
}

unsafe impl Sync for TT {}

impl TT {
    pub const fn new() -> TT {
        TT {
            table: UnsafeCell::new(Vec::new()),
        }
    }

    pub fn resize(&self, size: u64) {
        unsafe {
            (*self.table.get()).resize(size as usize, None);
        }
    }

    pub fn get(&self, key: Key) -> Option<Entry> {
        unsafe {
            let index = key.val() % self.size();
            (&(*self.table.get())).get(index as usize).unwrap().clone()
        }
    }

    pub fn set(&self, entry: Entry) {
        unsafe {
            let index = entry.key.val() % self.size();
            (&mut (*self.table.get()))[index as usize] = Some(entry);
        }
    }

    pub fn size(&self) -> u64 {
        unsafe { (*self.table.get()).len() as u64 }
    }

    /// Non null entries & Table Size
    pub fn load_factor(&self) -> (u64, u64) {
        let mut entry_count = 0;
        for index in unsafe { 0..(*self.table.get()).len() } {
            if unsafe { (&(*self.table.get())).get(index).unwrap().is_some() } {
                entry_count += 1;
            }
        }
        (entry_count, self.size())
    }
}

/// Alignment:
/// Key: u64 -> 8 bytes
/// Score: i32 -> 4 bytes
/// Mv: Option<NonZeroU32> -> 4 bytes
/// Depth: u8 -> 1 byte
/// Node_type: u8 -> 1 byte
/// Total: 18 bytes (This should probably be used better since its aligned to 24)
#[derive(Clone)]
pub struct Entry {
    pub key: Key,
    pub score: i32,
    pub mv: Mv,
    pub depth: u8,
    pub node_type: EntryType,
}

impl Entry {
    pub fn new(key: Key, score: i32, mut mv: Mv, depth: u8, node_type: EntryType) -> Entry {
        mv.sanitize_tt();
        Entry {
            key,
            score,
            mv,
            depth,
            node_type,
        }
    }
}

#[derive(Clone, PartialEq, Copy)]
pub enum EntryType {
    Upper,
    Lower,
    Exact,
}

#[derive(Clone, PartialEq, Default, Eq, Copy, Debug)]
pub struct Key(u64);

impl Key {
    pub fn new(p: &pos::Pos) -> Key {
        let mut key = Key(0);

        for (sq, piece) in p.piece_iter().enumerate() {
            if let Some(p) = piece {
                key.piece(sq as u8, p);
            }
        }

        if p.clr().is_black() {
            key.color();
        }

        let castle = p.castle();
        if castle.wk {
            key.castle(Clr::White, true);
        }
        if castle.wq {
            key.castle(Clr::White, false);
        }
        if castle.bk {
            key.castle(Clr::Black, true);
        }
        if castle.bq {
            key.castle(Clr::Black, false);
        }

        if let Some(file) = p.ep() {
            key.en_passant(file);
        }
        debug_assert!(key.val() != 0);
        key
    }

    pub fn new_from(val: u64) -> Key {
        Key(val)
    }

    pub fn val(&self) -> u64 {
        self.0
    }

    pub fn is_null(&self) -> bool {
        self.val() == 0
    }

    pub fn color(&mut self) {
        self.0 ^= unsafe { BLACK };
    }

    pub fn en_passant(&mut self, file: u8) {
        self.0 ^= unsafe { EN_PASSANT[file as usize] }
    }

    pub fn piece(&mut self, sq: u8, piece: ClrPiece) {
        let sq = sq as usize;
        self.0 ^= match piece {
            ClrPiece::WPawn => unsafe { PAWN[sq] },
            ClrPiece::WKnight => unsafe { KNIGHT[sq] },
            ClrPiece::WBishop => unsafe { BISHOP[sq] },
            ClrPiece::WRook => unsafe { ROOK[sq] },
            ClrPiece::WQueen => unsafe { QUEEN[sq] },
            ClrPiece::WKing => unsafe { KING[sq] },

            ClrPiece::BPawn => unsafe { BPAWN[sq] },
            ClrPiece::BKnight => unsafe { BKNIGHT[sq] },
            ClrPiece::BBishop => unsafe { BBISHOP[sq] },
            ClrPiece::BRook => unsafe { BROOK[sq] },
            ClrPiece::BQueen => unsafe { BQUEEN[sq] },
            ClrPiece::BKing => unsafe { BKING[sq] },
        }
    }

    pub fn castle(&mut self, clr: Clr, king_side: bool) {
        self.0 ^= match (clr, king_side) {
            (Clr::White, true) => unsafe { CASTLE[0] },
            (Clr::White, false) => unsafe { CASTLE[1] },
            (Clr::Black, true) => unsafe { CASTLE[2] },
            (Clr::Black, false) => unsafe { CASTLE[3] },
        }
    }
}

// Zobrist Keys Values
static mut PAWN: [u64; 64] = [0; 64];
static mut KNIGHT: [u64; 64] = [0; 64];
static mut BISHOP: [u64; 64] = [0; 64];
static mut ROOK: [u64; 64] = [0; 64];
static mut QUEEN: [u64; 64] = [0; 64];
static mut KING: [u64; 64] = [0; 64];

static mut BPAWN: [u64; 64] = [0; 64];
static mut BKNIGHT: [u64; 64] = [0; 64];
static mut BBISHOP: [u64; 64] = [0; 64];
static mut BROOK: [u64; 64] = [0; 64];
static mut BQUEEN: [u64; 64] = [0; 64];
static mut BKING: [u64; 64] = [0; 64];

static mut BLACK: u64 = 0;
static mut EN_PASSANT: [u64; 8] = [0; 8];
static mut CASTLE: [u64; 4] = [0; 4];

pub fn init_zobrist_keys() {
    let mut rng = rand::rng();
    let mut keys = [[0; 64]; 12];
    for i in 0..12 {
        for j in 0..64 {
            keys[i][j] = rng.next_u64();
        }
    }

    unsafe {
        PAWN = keys[0];
        KNIGHT = keys[1];
        BISHOP = keys[2];
        ROOK = keys[3];
        QUEEN = keys[4];
        KING = keys[5];

        BPAWN = keys[6];
        BKNIGHT = keys[7];
        BBISHOP = keys[8];
        BROOK = keys[9];
        BQUEEN = keys[10];
        BKING = keys[11];
    }

    let mut ep = [0; 8];
    for i in 0..8 {
        ep[i] = rng.next_u64();
    }
    unsafe { EN_PASSANT = ep }

    let mut castle = [0; 4];
    for i in 0..4 {
        castle[i] = rng.next_u64();
    }
    unsafe { CASTLE = castle }

    unsafe { BLACK = rng.next_u64() }
}
