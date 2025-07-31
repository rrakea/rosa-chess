use crate::{board::Board, table};
// Iternal representation of the current position.
// Using a hybrid approach of both bitboards and square centric

// w/b for color & p/n/b/r/q/k for the pieces using the common abbreviations
pub const PAWN: i8 = 1;
pub const BISHOP: i8 = 2;
pub const KNIGHT: i8 = 3;
pub const ROOK: i8 = 4;
pub const QUEEN: i8 = 5;
pub const KING: i8 = 6;

pub const BPAWN: i8 = -1;
pub const BBISHOP: i8 = -2;
pub const BKNIGHT: i8 = -3;
pub const BROOK: i8 = -4;
pub const BQUEEN: i8 = -5;
pub const BKING: i8 = -6;

pub const PIECE_VAL_ARRAY: [i8; 12] = [
    PAWN, BISHOP, KNIGHT, ROOK, QUEEN, KING, BPAWN, BBISHOP, BKNIGHT, BROOK, BQUEEN, BKING,
];

#[derive(Clone, Debug)]
pub struct Pos {
    // Bitboard centric layout
    // The boardarray is build like this:
    // 0 -> wpawn, 1 -> wbishop..
    // 6 -> bpawn, 7 -> bbishop..
    boards: [Board; 12],

    pub full: Board,

    // Square centric representation
    // Using the consts defined above
    sq: [i8; 64],

    pub key: table::Key,

    // Extra Data
    // Encoded like this: castling:  b queen, b king, w queen, b king; en_passant file;
    data: u8,

    // Active player (1 -> white; -1 -> black)
    pub active: i8,
}

impl Pos {
    pub fn new(
        sq: [i8; 64],
        active: i8,
        is_ep: bool,
        ep_file: u8,
        w_castle: (bool, bool),
        b_castle: (bool, bool),
    ) -> Pos {
        let mut boards = [Board::new(0); 12];
        for (sq, piece) in sq.into_iter().enumerate() {
            if piece != 0 {
                boards[calc_index(piece)].toggle(sq as u8);
            }
        }
        let mut newp = Pos {
            boards,
            sq,
            data: 0,
            active,
            full: Board::new(0),
            key: table::Key::default(),
        };
        newp.gen_new_full();
        newp.gen_new_data(is_ep, ep_file, w_castle, b_castle);
        newp.gen_new_key();
        newp
    }

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

    pub fn piece(&self, piece: i8) -> &Board {
        let index = calc_index(piece);
        &self.boards[index]
    }

    pub fn piece_at_sq(&self, sq: u8) -> i8 {
        self.sq[sq as usize]
    }

    pub fn piece_toggle(&mut self, piece: i8, sq: u8) {
        if piece == 0 {
            return;
        }
        self.sq[sq as usize] = piece;
        self.boards[calc_index(piece)].toggle(sq);
        self.key.piece(sq, piece);
    }

    pub fn piece_iter(&self) -> impl Iterator<Item = i8> {
        self.sq.into_iter()
    }

    pub fn prittify(&self) -> String {
        let mut str = String::new();
        for b in self.boards {
            str += format!("{}\n", b.prittify()).as_str();
        }
        str += format!("{}\n", self.full.prittify()).as_str();
        str += format!("Sq array: {:?}\n", self.sq).as_str();
        str += format!("Data: {:#010b}", self.data).as_str();
        str
    }

    pub fn gen_new_key(&mut self) {
        self.key = table::Key::new(self)
    }

    pub fn gen_new_full(&mut self) {
        let mut full = 0;
        for board in self.boards {
            full |= board.val();
        }
        self.full = Board::new(full);
    }

    pub fn gen_new_data(
        &mut self,
        is_ep: bool,
        ep_file: u8,
        w_castle: (bool, bool),
        b_castle: (bool, bool),
    ) {
        let mut data: u8 = 0;
        if is_ep {
            data |= 0b0000_1000;
            data |= ep_file;
        }
        if w_castle.0 {
            data |= 0b0001_0000;
        }
        if w_castle.1 {
            data |= 0b0010_0000;
        }
        if b_castle.0 {
            data |= 0b0100_0000;
        }
        if b_castle.1 {
            data |= 0b1000_0000;
        }
        self.data = data;
    }
}

fn calc_index(piece: i8) -> usize {
    let mut index = piece;
    if index < 0 {
        index = -index + 6;
    }
    // Since our pieces start at 1 but the array at 0
    index -= 1;
    if index < 0 || index >= 12 {
        scream!(
            "Wrong index in calc_index(), index: {}, piece: {}",
            index,
            piece
        );
    }
    index as usize
}
