#[derive(Clone, Copy, Debug)]
pub struct Board(u64);

impl Board {
    pub fn new(val: u64) -> Board {
        Board(val)
    }

    pub fn get_val(&self) -> u64 {
        self.0
    }

    pub fn get_ones(&self) -> Vec<u8> {
        let mut bb = self.get_val();
        let mut ones: Vec<u8> = Vec::new();
        let mut lsb;

        while bb != 0 {
            lsb = bb.trailing_zeros();
            ones.push(lsb as u8);
            bb &= bb - 1;
        }
        ones
    }

    pub fn get_ones_single(&self) -> u8 {
        self.0.trailing_zeros() as u8
    }

    pub fn set<T: Into<u64>>(&mut self, bit: T) {
        self.0 |= 1 << bit.into();
    }

    pub fn unset<T: Into<u64>>(&mut self, bit: T) {
        self.0 &= !(1 << bit.into());
    }

    pub fn set_all<T: Into<u64>>(&mut self, bits: Vec<T>) {
        for b in bits {
            self.0 |= 1 << b.into();
        }
    }

    pub fn unset_all<T: Into<u64>>(&mut self, bits: Vec<T>) {
        for b in bits {
            self.0 &= !(1 << b.into());
        }
    }
}
