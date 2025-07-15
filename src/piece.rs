// Currently not used, would require a lot a refactoring
// Hopefully in 2.0

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
}
