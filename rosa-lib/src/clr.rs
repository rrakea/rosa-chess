#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct Clr(bool);

impl Clr {
    pub fn as_bool(&self) -> bool {
        self.0
    }

    pub fn as_i8(&self) -> i8 {
        match self.0 {
            true => -1,
            false => 1,
        }
    }

    pub fn flip(&self) -> Clr {
        Clr(!self.0)
    }

    pub fn white() -> Clr {
        Clr(true)
    }

    pub fn black() -> Clr {
        Clr(false)
    }

    pub fn is_white(&self) -> bool {
        self.0
    }

    pub fn is_black(&self) -> bool {
        !self.0
    }
}

impl std::fmt::Display for Clr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let clr = if self.0 { "w" } else { "b" };
        write!(f, "{clr}")
    }
}
