use crate::board::Board;
use crate::clr::Clr;
use crate::piece::*;
use crate::tt;

#[derive(Clone) ]
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
        let mut boards = [Board::new(); 12];
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
            full: Board::new(),
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
        debug_assert!((0..64).contains(&sq));
        self.sq[sq as usize]
    }

    pub fn piece_toggle(&mut self, piece: ClrPiece, sq: u8) {
        self.sq[sq as usize] = match self.sq[sq as usize] {
            ClrPieceOption::None => ClrPieceOption::Some(piece),
            ClrPieceOption::Some(p) => {
                debug_assert_eq!(
                    p, piece,
                    "Tried toggling of an incorrect piece. Piece at sq: {}, Input Piece: {}",
                    p, piece
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

        if castle.wk {
            data |= 0b0001_0000;
        }
        if old_castle.wk != castle.wk {
            self.key.castle(Clr::White, true);
        }

        if castle.wq {
            data |= 0b0010_0000;
        }
        if castle.wq != old_castle.wq {
            self.key.castle(Clr::White, false);
        }

        if castle.bk {
            data |= 0b0100_0000;
        }
        if castle.bk != old_castle.bk {
            self.key.castle(Clr::Black, true);
        }

        if castle.bq {
            data |= 0b1000_0000;
        }
        if castle.bq != old_castle.bq {
            self.key.castle(Clr::Black, false);
        }

        self.data = data;
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

        if p1.data != p2.data {
            report.push_str(format!("Data mismatch: {:08b}, {:08b}", p1.data, p2.data).as_str());
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
    fn default() -> Self {
        Pos {
            boards: [Board::default(); 12],
            full: Board::default(),
            sq: [None; 64],
            key: tt::Key::default(),
            data: 0,
            clr: Clr::default()

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
