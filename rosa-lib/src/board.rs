//! # Bitboards
/// Since chess boards have 64 squares we can abuse 64 bit unsigned integers (bitboards) to represent where the pieces are.
/// Since bitboards have one bit of information for each square we have to save a bitboard for each piece & color.
/// Using bitboards not only speeds up but also optimizes the memory layout of the position struct.  
/// The main speed up comes from being able to quickly use bitwise operators for a ton of different operations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Board(u64);

impl Board {
    pub fn new() -> Board {
        Board(0)
    }

    pub fn new_from(val: u64) -> Board {
        Board(val)
    }

    pub fn val(&self) -> u64 {
        self.0
    }

    /// Gets on the positions where the bitboard is 1.  
    /// Preinitializes a vector on cap 8, since there will 99.9% of the time not be more than
    /// 8 instances of the same piece on the board
    #[inline(always)]
    pub fn get_ones(&self) -> Vec<u8> {
        let mut bb = self.val();
        let mut ones: Vec<u8> = Vec::with_capacity(8);
        let mut lsb;

        while bb != 0 {
            lsb = bb.trailing_zeros();
            ones.push(lsb as u8);
            bb &= bb - 1;
        }

        ones
    }

    pub fn at(&self, sq: u8) -> bool {
        (self.0 >> sq) & 1u64 == 1u64
    }

    /// Used when we are sure there can be only one piece
    /// i.e. Kings
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
        self.0 == 0
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
