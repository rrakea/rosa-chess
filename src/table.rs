use crate::mv::mv::Mv;
use crate::pos;
use rand::RngCore;

#[derive(Default)]
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

#[derive(Clone, PartialEq, Default, Eq, Copy, Debug)]
pub struct Key(u64);

impl Key {
    pub fn new(p: &pos::Pos) -> Key {
        let mut key = Key(0);
        for (sq, piece) in p.piece_iter().enumerate() {
            key.piece(sq as u8, piece);
        }
        //println!("After pieces: {}", key.val());

        if p.active == -1 {
            key.color();
            //println!("Color: {}", key.val());
        }

        let wc = p.castling(1);
        let bc = p.castling(-1);
        if wc.0 {
            key.castle(1, true);
            //println!("WC: {}", key.val());
        }
        if wc.1 {
            key.castle(1, false);
            //println!("WQ: {}", key.val());
        }
        if bc.0 {
            key.castle(-1, true);
            //println!("BK{}", key.val());
        }
        if bc.1 {
            key.castle(-1, false);
            //println!("BQ{}", key.val());
        }

        if p.is_en_passant() {
            key.en_passant(p.en_passant_file());
            //println!("EP{}", key.val());
        }

        //println!("END{}", key.val());
        if key.val() == 0 {
            panic!();
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

    unsafe {
        PAWN = [rng.next_u64(); 64];
        KNIGHT = [rng.next_u64(); 64];
        BISHOP = [rng.next_u64(); 64];
        ROOK = [rng.next_u64(); 64];
        QUEEN = [rng.next_u64(); 64];
        KING = [rng.next_u64(); 64];

        BPAWN = [rng.next_u64(); 64];
        BKNIGHT = [rng.next_u64(); 64];
        BBISHOP = [rng.next_u64(); 64];
        BROOK = [rng.next_u64(); 64];
        BQUEEN = [rng.next_u64(); 64];
        BKING = [rng.next_u64(); 64];
    }

    unsafe { EN_PASSANT = [rng.next_u64(); 8] }
    unsafe { CASTLE = [rng.next_u64(); 4] }
    unsafe { BLACK = rng.next_u64() }
}
