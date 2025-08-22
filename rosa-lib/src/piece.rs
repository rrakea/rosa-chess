use super::clr::Clr;

#[repr(i8)]
#[derive(Clone, Copy)]
pub enum Piece {
    WPawn = 1,
    WKnight = 2,
    WBishop = 3,
    WRook = 4,
    WQueen = 5,
    WKing = 6,

    BPawn = -1,
    BKnight = -2,
    BBishop = -3,
    BRook = -4,
    BQueen = -5,
    BKing = -6,
}

#[repr(i8)]
#[derive(Clone, Copy)]
pub enum Piece2 {
    WPawn(bool),
    WKnight(bool),
    WBishop(bool),
    WRook(bool),
    WQueen(bool),
    WKing(bool),

    BPawn(bool),
    BKnight(bool),
    BBishop(bool),
    BRook(bool),
    BQueen(bool),
    BKing(bool),
}

impl Piece {
    pub fn from_i8(val: i8) -> Piece {
        unsafe { std::mem::transmute::<i8, Piece>(val) }
    }

    pub fn flip(&self) -> Piece {
        Self::from_i8(-self.val())
    }

    pub fn val(&self) -> i8 {
        *self as i8
    }

    // King & Pawn last because promotion
    // only get 4 bits to encode
    pub fn to_mask(&self) -> u32 {
        0
    }

    pub fn clr(&self) -> Clr {
        println!("{}", std::mem::size_of::<Piece>());
        if self.val() > 0 {
            Clr::white()
        } else {
            Clr::black()
        }
    }

    pub fn to_clr(&mut self, clr: Clr) -> Piece {
        let res = match (clr.content(), self.clr().content()) {
            (true, true) => self.val(),
            (false, false) => self.val(),
            (true, false) => -self.val(),
            (false, true) => -self.val(),
        };
        Piece::from_i8(res)
    }
}
