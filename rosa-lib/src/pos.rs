//! # Position Representation
//! To keep track of the state of a chess position during search a accurate and fast board representation is needed.
//! The data it needs to keep track of include: the position of all the pieces, castling rights, side to move and en passant rights.
//! Different techniques can be used to represent the pieces, bitboards being the most popular one.
//! Rosa Chess uses a hybrid approach of both bitboards for every piece and a piece table representation.
//! Both of them are optimal for different tasks
//! (Bitboards for move generation, piece tables for checking for checks & promotions)
//! Since this struct is only constructed once you dont need to particuarly optimize for memory layout.
//! Instead speed of access and storing is key.

use crate::board::Board;
use crate::piece::*;
use crate::tt;

#[derive(Clone)]
pub struct Pos {
    // Bitboard centric layout
    // The boardarray is build like this:

    // 0 -> wpawn, 1 -> wbishop..
    // 6 -> bpawn, 7 -> bbishop..
    boards: [Board; 12],

    full: Board,

    // Square centric representation
    // Using the consts defined above
    sq: [ClrPieceOption; 64],

    key: tt::Key,

    clr: Clr,
    ep: Option<u8>,
    castle: Castling,
    pub halfmove: u8
}

#[derive(PartialEq, Eq, Copy, Clone, Default, Debug)]
pub struct Castling {
    pub wk: bool,
    pub wq: bool,
    pub bk: bool,
    pub bq: bool,
}

impl Pos {
    pub fn new(
        sq: [ClrPieceOption; 64],
        clr: Clr,
        is_ep: bool,
        ep_file: u8,
        castle: Castling,
    ) -> Pos {
        let mut boards = [Board::new(); 12];
        for (sq, piece) in sq.into_iter().enumerate() {
            if let Some(p) = piece {
                boards[p.index()].toggle(sq as u8);
            }
        }

        let mut newp = Pos {
            boards,
            sq,
            clr,
            castle,
            full: Board::new(),
            key: tt::Key::default(),
            ep: is_ep.then(|| ep_file),
            halfmove: 0
        };

        newp.gen_new_full();
        newp.gen_new_key();
        newp
    }

    pub fn key(&self) -> tt::Key {
        self.key
    }

    pub fn castle(&self) -> Castling {
        self.castle
    }

    pub fn ep(&self) -> Option<u8> {
        self.ep
    }

    pub fn clr(&self) -> Clr {
        self.clr
    }

    pub fn full(&self) -> Board {
        self.full
    }

    pub fn piece(&self, piece: ClrPiece) -> Board {
        self.boards[piece.index()]
    }

    pub fn piece_at_sq(&self, sq: u8) -> ClrPieceOption {
        debug_assert!((0..64).contains(&sq));
        self.sq[sq as usize]
    }

    pub fn board_empty_at(&self, sq: u8) -> bool {
        self.full.at(sq) == false
    }

    pub fn piece_toggle(&mut self, piece: ClrPiece, sq: u8) {
        self.sq[sq as usize] = match self.sq[sq as usize] {
            ClrPieceOption::None => ClrPieceOption::Some(piece),
            ClrPieceOption::Some(p) => {
                debug_assert_eq!(
                    p, piece,
                    "Tried toggling of an incorrect piece. Piece at sq {sq}: {}, Input Piece: {}, Pos:\n{}",
                    p, piece, self
                );
                ClrPieceOption::None
            }
        };

        self.full.toggle(sq);
        self.boards[piece.index()].toggle(sq);
        self.key.piece(sq, piece);
    }

    pub fn piece_iter(&self) -> impl Iterator<Item = ClrPieceOption> {
        self.sq.into_iter()
    }

    pub fn flip_color(&mut self) {
        self.clr = self.clr.flip();
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
        self.full = Board::new_from(full);
    }

    pub fn set_ep(&mut self, ep: Option<u8>) {
        if let Some(old) = self.ep {
            self.key.en_passant(old);
        }

        if let Some(new) = ep {
            self.key.en_passant(new);
        }

        self.ep = ep;
    }

    pub fn set_castling(&mut self, c: Castling) {
        if self.castle.wk != c.wk {
            self.key.castle(Clr::White, true);
        }
        if self.castle.wq != c.wq {
            self.key.castle(Clr::White, false);
        }
        if self.castle.bk != c.bk {
            self.key.castle(Clr::Black, true);
        }
        if self.castle.bq != c.bq {
            self.key.castle(Clr::Black, false);
        }

        self.castle = c;
    }

    pub fn is_default(&self) -> bool {
        self.full.empty()
    }

    pub fn debug_key_mismatch(p1: &Pos, p2: &Pos) -> String {
        let mut report = String::new();
        if p1.key == p2.key {
            report.push_str("Keys not actually mismatched\n");
        } else {
            report.push_str("Keys mismatched\n");
        }

        for i in 0..12 {
            let b1 = p1.boards[i];
            let b2 = p2.boards[i];
            if b1 != b2 {
                report
                    .push_str(format!("Two boards at position {i} mismatch: {b1}, {b2}").as_str());
            }
        }

        if p1.castle != p2.castle {
            report.push_str(
                format!(
                    "Missmatch in castling data: {:?}, {:?}",
                    p1.castle, p2.castle
                )
                .as_str(),
            );
        }

        if p1.ep != p2.ep {
            report.push_str(format!("Missmatch in ep data: {:?}, {:?}", p1.ep, p2.ep).as_str());
        }

        if p1.clr != p2.clr {
            report.push_str("Color mismatch");
        }

        if p1.full != p2.full {
            report.push_str(format!("Full mismatch: {}, {}", p1.full, p2.full).as_str());
        }

        for sq in 0..64 {
            let piece1 = p1.piece_at_sq(sq);
            let piece2 = p2.piece_at_sq(sq);
            if piece1 != piece2 {
                report.push_str(
                    format!("Piece mismatch at sq: {sq}, {:?}, {:?}", piece1, piece2).as_str(),
                );
            }
        }

        report
    }
}

impl Default for Pos {
    // You cant define default for a type alias ahhh
    fn default() -> Self {
        Pos {
            boards: [Board::default(); 12],
            full: Board::default(),
            sq: [None; 64],
            key: tt::Key::default(),
            castle: Castling::default(),
            clr: Clr::default(),
            ep: Option::default(),
            halfmove: 0
        }
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
        board += format!("To move: {}\n", self.clr).as_str();
        board += format!("Castling right: {:?}\n", self.castle()).as_str();
        board += format!("En passant file: {:?}\n", self.ep()).as_str();
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
        str += format!("{:?}\n", self.castle).as_str();
        str += format!("{:?}\n", self.ep).as_str();
        str += format!("Color: {}", self.clr).as_str();

        write!(f, "{}", str)
    }
}
