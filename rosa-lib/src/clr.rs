pub struct Clr(bool);

impl Clr {
    pub fn content(&self) -> bool {
        self.0
    }

    pub fn val(&self) -> i8 {
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
}
