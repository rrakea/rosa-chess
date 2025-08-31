use crate::board::Board;
use crate::clr::Clr;
use crate::piece::*;
use crate::tt;

#[derive(Clone)]
pub struct Pos {
    // Bitboard centric layout
    // The boardarray is build like this:

    // 0 -> wpawn, 1 -> wbishop..
    // 6 -> bpawn, 7 -> bbishop..
    boards: [Board; 12],

    pub full: Board,

    // Square centric representation
    // Using the consts defined above
    sq: [ClrPieceOption; 64],

    key: tt::Key,

    // Extra Data
    // Encoded like this: castling:  b queen, b king, w queen, b king; en_passant file;
    data: u8,

    pub clr: Clr,
}

impl Pos {
    pub fn new(
        sq: [ClrPieceOption; 64],
        clr: Clr,
        is_ep: bool,
        ep_file: u8,
        castle: CastleData,
    ) -> Pos {
        let mut boards = [Board::new(0); 12];
        for (sq, piece) in sq.into_iter().enumerate() {
            if let Some(p) = piece {
                boards[p.index()].toggle(sq as u8);
            }
        }

        let mut newp = Pos {
            boards,
            sq,
            data: 0,
            clr,
            full: Board::new(0),
            key: tt::Key::default(),
        };

        newp.gen_new_data(is_ep, ep_file, castle);
        newp.gen_new_full();
        newp.gen_new_key();
        newp
    }

    pub fn is_en_passant(&self) -> bool {
        (self.data & 0b0000_1000) != 0
    }

    pub fn en_passant_file(&self) -> u8 {
        self.data & 0b0000_0111
    }

    pub fn key(&self) -> tt::Key {
        self.key
    }

    // -> kingside, queenside
    pub fn castle_data(&self) -> CastleData {
        CastleData {
            wk: self.data & 0b0001_0000 > 0,
            wq: self.data & 0b0010_0000 > 0,
            bk: self.data & 0b0100_0000 > 0,
            bq: self.data & 0b1000_0000 > 0,
        }
    }

    pub fn can_castle(&self, clr: Clr) -> (bool, bool) {
        let data = self.castle_data();
        if clr.is_white() {
            (data.wk, data.wq)
        } else {
            (data.bk, data.bq)
        }
    }

    pub fn piece(&self, piece: ClrPiece) -> &Board {
        &self.boards[piece.index()]
    }

    pub fn piece_at_sq(&self, sq: u8) -> ClrPieceOption {
        self.sq[sq as usize]
    }

    pub fn piece_toggle(&mut self, piece: ClrPiece, sq: u8) {
        self.sq[sq as usize] = match self.sq[sq as usize] {
            ClrPieceOption::None => ClrPieceOption::Some(piece),
            ClrPieceOption::Some(_p) => ClrPieceOption::None,
        };

        self.full.toggle(sq);
        self.boards[piece.index()].toggle(sq);
        self.key.piece(sq, piece);
    }

    pub fn piece_iter(&self) -> impl Iterator<Item = ClrPieceOption> {
        self.sq.into_iter()
    }

    pub fn flip_color(&mut self) {
        self.clr.flip();
        self.key.color();
    }

    pub fn gen_new_key(&mut self) {
        self.key = tt::Key::new(self)
    }

    fn gen_new_full(&mut self) {
        let mut full = 0;
        for board in self.boards {
            full |= board.val();
        }
        self.full = Board::new(full);
    }

    pub fn gen_new_data(&mut self, is_ep: bool, ep_file: u8, castle: CastleData) {
        // Unset the old ep key
        if self.is_en_passant() {
            self.key.en_passant(self.en_passant_file());
        }

        let mut data: u8 = 0;
        if is_ep {
            data |= 0b0000_1000;
            data |= ep_file;
            self.key.en_passant(ep_file);
        }

        let old_castle = self.castle_data();

        // Since we cant regain castling rights we only have to
        // unset the key if we previously had them
        if castle.wk {
            data |= 0b0001_0000;
        } else if old_castle.wk {
            self.key.castle(Clr::White, true);
        }
        if castle.wq {
            data |= 0b0010_0000;
        } else if old_castle.wq {
            self.key.castle(Clr::White, false);
        }
        if castle.bk {
            data |= 0b0100_0000;
        } else if old_castle.bk {
            self.key.castle(Clr::Black, true);
        }
        if castle.bq {
            data |= 0b1000_0000;
        } else if old_castle.bq {
            self.key.castle(Clr::Black, false);
        }
        self.data = data;
    }
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ranks = Vec::new();
        let mut buf = Vec::new();

        for (sq, piece) in self.sq.iter().enumerate() {
            if sq % 8 == 0 {
                ranks.push(buf);
                buf = Vec::new();
            }
            buf.push(piece);
        }
        ranks.push(buf);

        let mut board = String::new();
        for rank in ranks.iter().rev() {
            for piece in rank {
                match piece {
                    Some(p) => board += p.to_string().as_str(),
                    None => board += " ",
                }
                board += "|"
            }
            board += "\n"
        }
        write!(f, "{}", board)
    }
}

impl std::fmt::Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for b in self.boards {
            str += format!("{}\n", b).as_str();
        }
        str += format!("{}\n", self.full).as_str();
        str += format!("Data: {:#010b}\n", self.data).as_str();
        str += format!("Color: {}", self.clr).as_str();

        write!(f, "{}", str)
    }
}

pub struct CastleData {
    pub wk: bool,
    pub wq: bool,
    pub bk: bool,
    pub bq: bool,
}
