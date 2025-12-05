//! # Color
//! Used for some optimizations with the piece type

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub enum Clr {
    #[default]
    White,
    Black,
}

impl Clr {
    pub fn as_sign(&self) -> i8 {
        match self {
            Clr::White => 1,
            Clr::Black => -1,
        }
    }

    pub fn flip(&self) -> Clr {
        match self {
            Clr::White => Clr::Black,
            Clr::Black => Clr::White,
        }
    }

    pub fn is_white(&self) -> bool {
        *self == Clr::White
    }

    pub fn is_black(&self) -> bool {
        *self == Clr::Black
    }
}

impl std::fmt::Display for Clr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let clr = if self.is_white() { "w" } else { "b" };
        write!(f, "{clr}")
    }
}
