use super::clr::Clr;

#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum PieceNull {
    Null,
    Piece(Piece)
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
        if self.val() > 0 {
            Clr::white()
        } else {
            Clr::black()
        }
    }

    pub fn to_clr(&self, clr: Clr) -> Piece {
        if clr != self.clr() {
            Piece::from_i8(-self.val())
        } else {
            *self
        }
    }

    pub fn norm(&self) -> Piece {
        self.to_clr(Clr::white())
    }

    pub fn is_null(&self) -> bool {
        *self != Piece::Null
    }
}
