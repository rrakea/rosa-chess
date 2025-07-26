use crate::board::Board;
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
    pub boards: [Board; 12],

    pub full: Board,

    // Square centric representation
    // Using the consts defined above
    pub sq: [i8; 64],

    // Extra Data
    // Encoded like this: castling:  b queen, b king, w queen, b king; en_passant file;
    pub data: u8,

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
                boards[calc_index(piece)].set(sq as u8);
            }
        }
        let data = gen_data(is_ep, ep_file, w_castle, b_castle);
        let mut newp = Pos {
            boards,
            sq,
            data,
            active,
            full: Board::new(0),
        };
        gen_full(&mut newp);
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

    pub fn piece_mut(&mut self, piece: i8) -> &mut Board {
        let index = calc_index(piece);
        &mut self.boards[index]
    }

    pub fn piece(&self, piece: i8) -> &Board {
        let index = calc_index(piece);
        &self.boards[index]
    }

    pub fn print(&self) {
        println!("Bitboards: ");
        for b in self.boards {
            println!("{}", b.prittify());
        }
        println!();
            println!("Full: {}", self.full.prittify());


        println!("Sq array: {:?}", self.sq);
        println!("Data: {:#066b}", self.data);
    }
}

pub fn gen_full(p: &mut Pos) {
    let mut full = 0;
    for board in p.boards {
        full |= board.val();
    }
    p.full = Board::new(full);
}

pub fn gen_data(is_ep: bool, ep_file: u8, w_castle: (bool, bool), b_castle: (bool, bool)) -> u8 {
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
    data
}

fn calc_index(piece: i8) -> usize {
    let mut index = piece;
    if index < 0 {
        index = -index + 6;
    }
    // Since our pieces start at 1 but the array at 0
    index -= 1;   
    index as usize
}
