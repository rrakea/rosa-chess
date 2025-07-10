use crate::mv::mv::Mv;
use crate::pos::pos;
use rand::RngCore;

pub struct TT {
    t: Vec<Entry>,
    size: u64,
}

#[derive(Clone, Default)]
pub struct Entry {
    key: Key,
    best: Mv,
    second: Mv,
    score: i8,
    depth: u8,
    node_type: i8, // -1 -> lower bound; 0 -> exact; 1 -> upper bound
    age: u8,
}

#[derive(Clone, PartialEq, Default)]
pub struct Key(u64);

impl TT {
    pub fn new(size: u64) -> TT {
        let table: Vec<Entry> = vec![Entry::default(); size as usize];
        TT { t: table, size }
    }

    pub fn get(&self, key: Key) -> Option<&Entry> {
        let index = key.get() % self.size;
        let entry = &self.t[index as usize];
        if entry.key == key {
            Some(&entry)
        } else {
            None
        }
    }

    pub fn set(&mut self, entry: Entry) {
        let index = entry.key.get() % self.size;
        self.t[index as usize] = entry;
    }
}

impl Key {
    pub fn new(p: &pos::Pos) -> Key {
        let mut key = Key(0);
        for (i, val) in p.sq.iter().enumerate() {
            key.0 ^= match *val {
                pos::WPAWN => unsafe { PAWN[i] },
                pos::WKNIGHT => unsafe { KNIGHT[i] },
                pos::WBISHOP => unsafe { BISHOP[i] },
                pos::WROOK => unsafe { ROOK[i] },
                pos::WQUEEN => unsafe { QUEEN[i] },
                pos::WKING => unsafe { KING[i] },

                pos::BPAWN => unsafe { BPAWN[i] },
                pos::BKNIGHT => unsafe { BKNIGHT[i] },
                pos::BBISHOP => unsafe { BBISHOP[i] },
                pos::BROOK => unsafe { BROOK[i] },
                pos::BQUEEN => unsafe { BQUEEN[i] },
                pos::BKING => unsafe { BKING[i] },

                //Since we are xor'ing with the key the inversion does nothing
                0 => !key.0,
                _ => panic!("Invalid Piece value: {}", *val),
            };
        }

        if p.active == -1 {
            key.color();
        }

        let wc = p.castling(1);
        let bc = p.castling(-1);
        if wc.0 {
            key.castle(1, true);
        }
        if wc.1 {
            key.castle(1, false)
        }
        if bc.0 {
            key.castle(-1, true);
        }
        if bc.1 {
            key.castle(-1, false);
        }

        if p.is_en_passant() {
            key.en_passant(p.en_passant_file());
        }

        key
    }

    pub fn get(&self) -> u64 {
        self.0
    }

    pub fn color(&mut self) {
        self.0 ^= unsafe { BLACK };
    }

    pub fn en_passant(&mut self, file: u8) {
        self.0 ^= unsafe { EN_PASSANT[file as usize] }
    }

    pub fn piece(&mut self, sq: u8, piece: i8) {
        let sq = sq as usize;
        self.0 ^= match piece {
            pos::WPAWN => unsafe { PAWN[sq] },
            pos::WKNIGHT => unsafe { KNIGHT[sq] },
            pos::WBISHOP => unsafe { BISHOP[sq] },
            pos::WROOK => unsafe { ROOK[sq] },
            pos::WQUEEN => unsafe { QUEEN[sq] },
            pos::WKING => unsafe { KING[sq] },

            pos::BPAWN => unsafe { BPAWN[sq] },
            pos::BKNIGHT => unsafe { BKNIGHT[sq] },
            pos::BBISHOP => unsafe { BBISHOP[sq] },
            pos::BROOK => unsafe { BROOK[sq] },
            pos::BQUEEN => unsafe { BQUEEN[sq] },
            pos::BKING => unsafe { BKING[sq] },
            _ => panic!("Invalid piece code for getting a castle hash: {}", piece),
        }
    }

    pub fn castle(&mut self, active: i8, king_side: bool) {
        self.0 ^= match (active, king_side) {
            (1, true) => unsafe { CASTLE[0] },
            (1, false) => unsafe { CASTLE[1] },
            (-1, true) => unsafe { CASTLE[2] },
            (-1, false) => unsafe { CASTLE[3] },
            _ => panic!("Invalid value in castle_hash: {}, {}", active, king_side),
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
        for j in 0..12 * 64 {
            keys[j][i] = rng.next_u64();
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
    unsafe { CASTLE = castle };

    unsafe { BLACK = rng.next_u64() }
}
