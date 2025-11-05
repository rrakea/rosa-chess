use crate::clr::Clr;
use crate::mv::Mv;
use crate::piece::*;
use crate::pos;

use rand::RngCore;
use std::cell::UnsafeCell;

#[derive(Default)]
pub struct TT {
    table: UnsafeCell<Vec<Entry>>,
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
            (*self.table.get()).resize(size as usize, Entry::default());
        }
    }

    pub fn get(&self, key: &Key) -> Entry {
        unsafe {
            let index = key.val() % self.size();
            (&(*self.table.get())).get(index as usize).unwrap().clone()
        }
    }

    pub fn checked_get(&self, key: &Key) -> Option<Entry> {
        let entry = self.get(key);
        if entry.is_null() {
            None
        } else {
            Some(entry)
        }
    }

    pub fn set(&self, entry: Entry) {
        unsafe {
            let index = entry.key.val() % self.size();
            (&mut (*self.table.get()))[index as usize] = entry;
        }
    }

    pub fn size(&self) -> u64 {
        unsafe { (*self.table.get()).len() as u64 }
    }

    pub fn usage(&self) -> (u64, u64, u64) {
        let mut entry_count = 0;
        let mut null_count = 0;
        for index in unsafe { 0..(*self.table.get()).len() } {
            let node_type = unsafe { (&(*self.table.get())).get(index).unwrap().node_type };
            if node_type == EntryType::Null {
                null_count += 1;
            } else {
                entry_count += 1;
            }
        }
        (entry_count, null_count, self.size())
    }
}

/*
    Alignment:
    Key: u64 -> 8 bytes
    Score: i32 -> 4 bytes
    mv: u16 -> 2 byte
    Depth: u8 -> 1 byte
    Node_type: i8 -> 1 byte

    => 16 Bytes/ 156 bit (-> No padding)
*/

#[derive(Clone)]
pub struct Entry {
    pub key: Key,
    pub score: i32,
    pub mv: Mv,
    pub depth: u8,
    pub node_type: EntryType,
}

#[derive(Clone, PartialEq, Copy)]
pub enum EntryType {
    Null,
    Upper,
    Lower,
    Exact,
}

impl Entry {
    pub fn new(key: Key, score: i32, mv: Mv, depth: u8, node_type: EntryType) -> Entry {
        Entry {
            key,
            score,
            mv,
            depth,
            node_type,
        }
    }

    pub fn is_null(&self) -> bool {
        self.node_type == EntryType::Null
    }
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            key: Key::new_from(0),
            score: 0,
            mv: Mv::null(),
            depth: 0,
            node_type: EntryType::Null,
        }
    }
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

        if p.clr.is_black() {
            key.color();
        }

        let castle = p.castle_data();
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

        if p.is_en_passant() {
            key.en_passant(p.en_passant_file());
        }

        if key.val() == 0 {
            panic!();
        }
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
