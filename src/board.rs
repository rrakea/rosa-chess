#[derive(Clone, Copy, Debug)]
pub struct Board(u64);

impl Board {
    pub fn new(val: u64) -> Board {
        Board(val)
    }

    pub fn val(&self) -> u64 {
        self.0
    }

    pub fn get_ones(&self) -> Vec<u8> {
        let mut bb = self.val();
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

    pub fn set(&mut self, bit: u8) {
        self.0 |= 1 << bit;
    }

    pub fn unset(&mut self, bit: u8) {
        self.0 &= !1 << bit;
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

    pub fn count(&self) -> u32 {
        self.val().count_ones()
    }

    pub fn empty(&self) -> bool {
        self.0 != 0
    }

    pub fn prittify(&self) -> String{
        let mut buf = Vec::new();
        let bit_str = format!("{:064b}", self.val());
        for rank in 0..8 {
            let start = rank * 8;
            let end = start + 8;
            let row: String = bit_str[start..end].chars().rev().collect();
            buf.push(row);
        }

        format!("Bit: {:064b}; Board:\n{}\n", self.val(), buf.join("\n"))
    }
}
