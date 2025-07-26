use crate::mv::mv::Mv;
use crate::pos;
use rand::RngCore;

pub struct TT {
    table: Vec<Entry>,
    size: u64,
}

impl TT {
    pub fn new(size: u64) -> TT {
        let table: Vec<Entry> = vec![Entry::default(); size as usize];
        TT { table, size }
    }

    pub fn get(&self, key: &Key) -> &Entry {
        let index = key.val() % self.size;
        &self.table[index as usize]
    }

    pub fn set(&mut self, entry: Entry) {
        let index = entry.key.val() % self.size;
        self.table[index as usize] = entry;
    }
}

/*
    Alignment:
    Key: u64 -> 8 bytes
    Score: i32 -> 4 bytes
    mv: u16 -> 2 byte
    Depth: u8 -> 1 byte
    Node_type: i8 -> 1 byte

    => 16 Bytes (-> No padding)
*/

#[derive(Clone, Default)]
pub struct Entry {
    pub key: Key,
    pub score: i32,
    pub mv: Mv,
    pub depth: u8,
    pub node_type: NodeType,
}

#[derive(Clone, Default, PartialEq)]
pub enum NodeType {
    #[default]
    Null,
    Upper,
    Lower,
    Exact,
}

impl Entry {
    pub fn new(key: Key, score: i32, mv: Mv, depth: u8, node_type: NodeType) -> Entry {
        Entry {
            key,
            score,
            mv,
            depth,
            node_type,
        }
    }
}

#[derive(Clone, PartialEq, Default, Eq, Copy)]
pub struct Key(u64);

impl Key {
    pub fn new(p: &pos::Pos) -> Key {
        let mut key = Key(0);
        for (i, val) in p.sq.iter().enumerate() {
            key.0 ^= match *val {
                pos::PAWN => unsafe { PAWN[i] },
                pos::KNIGHT => unsafe { KNIGHT[i] },
                pos::BISHOP => unsafe { BISHOP[i] },
                pos::ROOK => unsafe { ROOK[i] },
                pos::QUEEN => unsafe { QUEEN[i] },
                pos::KING => unsafe { KING[i] },

                pos::BPAWN => unsafe { BPAWN[i] },
                pos::BKNIGHT => unsafe { BKNIGHT[i] },
                pos::BBISHOP => unsafe { BBISHOP[i] },
                pos::BROOK => unsafe { BROOK[i] },
                pos::BQUEEN => unsafe { BQUEEN[i] },
                pos::BKING => unsafe { BKING[i] },

                // x ^ 0 = x
                0 => 0,
                _ => scream!("Invalid Piece value new key call: {}", *val),
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

    pub fn piece(&mut self, sq: u8, piece: i8) {
        let sq = sq as usize;
        self.0 ^= match piece {
            pos::PAWN => unsafe { PAWN[sq] },
            pos::KNIGHT => unsafe { KNIGHT[sq] },
            pos::BISHOP => unsafe { BISHOP[sq] },
            pos::ROOK => unsafe { ROOK[sq] },
            pos::QUEEN => unsafe { QUEEN[sq] },
            pos::KING => unsafe { KING[sq] },

            pos::BPAWN => unsafe { BPAWN[sq] },
            pos::BKNIGHT => unsafe { BKNIGHT[sq] },
            pos::BBISHOP => unsafe { BBISHOP[sq] },
            pos::BROOK => unsafe { BROOK[sq] },
            pos::BQUEEN => unsafe { BQUEEN[sq] },
            pos::BKING => unsafe { BKING[sq] },
            0 => 0,
            _ => scream!("Invalid piece code for getting a piece hash: {}", piece),
        }
    }

    pub fn castle(&mut self, active: i8, king_side: bool) {
        self.0 ^= match (active, king_side) {
            (1, true) => unsafe { CASTLE[0] },
            (1, false) => unsafe { CASTLE[1] },
            (-1, true) => unsafe { CASTLE[2] },
            (-1, false) => unsafe { CASTLE[3] },
            _ => scream!("Invalid value in castle_hash: {}, {}", active, king_side),
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
    unsafe { CASTLE = castle };

    unsafe { BLACK = rng.next_u64() }
}
