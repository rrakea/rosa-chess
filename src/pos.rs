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
    // Bitboard centric layout
    pub boards: BoardArray,

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
        let boards = BoardArray::new(sq);
        let data = gen_data(is_ep, ep_file, w_castle, b_castle);
        Pos {
            boards,
            sq,
            data,
            active,
        }
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
}

fn gen_data(is_ep: bool, ep_file: u8, w_castle: (bool, bool), b_castle: (bool, bool)) -> u8 {
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

#[derive(Clone, Debug)]
pub struct BoardArray {
    // The boardarray is build like this:
    // 0 -> wpawn, 1 -> wbishop..
    // 6 -> bpawn, 7 -> bbishop..
    boards: [Board; 12],

    // Full board bitboard
    pub full: Board,
}

impl BoardArray {
    pub fn new(sq: [i8; 64]) -> BoardArray {
        let mut board_array = BoardArray {
            boards: [Board::new(0); 12],
            full: Board::new(0),
        };
        for (sq, piece) in sq.into_iter().enumerate() {
            if piece != 0 {
                let mut current_board = board_array.get(piece);
                current_board.set(sq as u8);
                board_array.set(piece, current_board);
            }
        }

        let mut full = 0;
        for board in &board_array.boards {
            full |= board.get_val();
        }
        board_array.full = Board::new(full);

        board_array
    }

    pub fn get(&self, piece: i8) -> Board {
        let index = calc_index(piece);
        self.boards[index]
    }

    pub fn set(&mut self, piece: i8, val: Board) {
        let index = calc_index(piece);
        self.boards[index] = val;
    }
}

fn calc_index(piece: i8) -> usize {
    let mut index = piece;
    if index < 0 {
        index = index * -1 + 6;
    }
    // Since our pieces start at 1 but the array at 0
    index -= 1;
    index as usize
}
