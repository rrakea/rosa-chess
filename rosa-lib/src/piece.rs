use crate::clr::Clr;

/*
    This module provides 3 enums for representing pieces.
    I chose 3 different ones, to make the api clear
    So when checking what piece occupies a square you will get
    a ClrPieceOption, so it is clear that it could be empty
    Piece is used for stuff where the color does not matter
    i.e. to which piece to promote to
*/

#[repr(i8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Piece {
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

#[repr(i8)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub enum ClrPiece {
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

pub type ClrPieceOption = Option<ClrPiece>;

impl ClrPiece {
    pub fn flip(&self) -> ClrPiece {
        Self::from(-self.val())
    }

    pub fn val(&self) -> i8 {
        *self as i8
    }

    // Returns a range from 0 to 11
    pub fn index(&self) -> usize {
        let mut index = self.val();
        if index < 0 {
            index = -index + 6;
        }
        // Since our pieces start at 1 but the array at 0
        index -= 1;
        debug_assert!((0..12).contains(&index));
        index as usize
    }

    pub fn clr(&self) -> Clr {
        if self.val() > 0 {
            Clr::White
        } else {
            Clr::Black
        }
    }

    pub fn as_clr(&self, clr: Clr) -> ClrPiece {
        if clr != self.clr() {
            ClrPiece::from(-self.val())
        } else {
            *self
        }
    }

    pub fn de_clr(&self) -> Piece {
        Piece::from(self.val().abs())
    }
}

impl std::fmt::Display for ClrPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece_str = match self {
            Self::WPawn => "\u{2659}",
            Self::WKnight => "\u{2658}",
            Self::WBishop => "\u{2657}",
            Self::WRook => "\u{2656}",
            Self::WQueen => "\u{2655}",
            Self::WKing => "\u{2654}",

            Self::BPawn => "\u{265F}",
            Self::BKnight => "\u{265E}",
            Self::BBishop => "\u{265D}",
            Self::BRook => "\u{265C}",
            Self::BQueen => "\u{265B}",
            Self::BKing => "\u{265A}",
        };

        write!(f, "{}", piece_str)
    }
}

impl Piece {
    pub fn val(self) -> i8 {
        self as i8
    }

    pub fn clr(&self, clr: Clr) -> ClrPiece {
        ClrPiece::from(self.val()).as_clr(clr)
    }

    // Compresses the piece into 2 bytes for storing the promoted piece
    // Knight(2) => 0, Queen(5) => 3
    pub fn compress_prom(&self) -> u32 {
        (self.val() - 2) as u32
    }

    pub fn decompress_prom(data: u32) -> Piece {
        Piece::from(data as i8 + 2)
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece = match self {
            Piece::Pawn => "p",
            Piece::Knight => "n",
            Piece::Bishop => "b",
            Piece::Rook => "r",
            Piece::Queen => "q",
            Piece::King => "k",
        };
        write!(f, "{piece}")
    }
}

impl std::fmt::Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece = match self {
            Piece::Pawn => "p",
            Piece::Knight => "n",
            Piece::Bishop => "b",
            Piece::Rook => "r",
            Piece::Queen => "q",
            Piece::King => "k",
        };
        write!(f, "{piece}")
    }
}

impl std::convert::From<i8> for ClrPiece {
    fn from(value: i8) -> Self {
        debug_assert!(value != 0 && value <= 6 && value >= -6, "Value: {value}");
        
        unsafe { std::mem::transmute(value) }
    }
}

impl std::convert::From<i8> for Piece {
    fn from(value: i8) -> Self {
        debug_assert!(value != 0 && value <= 6 && value >= -6, "Value: {value}");
        unsafe { std::mem::transmute(value) }
    }
}
