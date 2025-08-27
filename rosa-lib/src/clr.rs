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
