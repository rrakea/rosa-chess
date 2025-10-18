#[derive(Clone, Copy, Debug)]
pub struct Board(u64);

impl Board {
    pub fn new() -> Board{
        Board(0)
    }

    pub fn new_from(val: u64) -> Board {
        Board(val)
    }

    pub fn val(&self) -> u64 {
        self.0
    }

    pub fn get_ones(&self) -> Vec<u8> {
        let mut bb = self.val();
        let count = bb.count_ones();
        let mut ones: Vec<u8> = Vec::with_capacity(count as usize);
        let mut lsb;

        for _i in 0..count {
            lsb = bb.trailing_zeros();
            ones.push(lsb as u8);
            bb &= bb - 1;
        }

        ones
    }

    pub fn get_ones_single(&self) -> u8 {
        self.0.trailing_zeros() as u8
    }

    pub fn toggle(&mut self, bit: u8) {
        debug_assert!(bit < 64, "Bit: {bit}");
        self.0 ^= 1 << bit;
    }

    pub fn toggle_all(&mut self, bits: Vec<u8>) {
        for b in bits {
            self.toggle(b);
        }
    }

    pub fn count(&self) -> u32 {
        self.val().count_ones()
    }

    pub fn empty(&self) -> bool {
        self.0 != 0
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = Vec::new();
        let bit_str = format!("{:064b}", self.val());
        for rank in 0..8 {
            let start = rank * 8;
            let end = start + 8;
            let row: String = bit_str[start..end].chars().rev().collect();
            buf.push(row);
        }

        write!(f, "{}\n", buf.join("\n"))
    }
}
