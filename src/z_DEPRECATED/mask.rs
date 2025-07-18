struct Mask(u64);
impl Mask {
    pub fn one_at(b: Vec<u8>) -> Mask{
        let mut mask = 0;
        for bit in b {
            mask |= 1 << bit;
        }
        Mask(mask)
    }

    pub fn zero_at(b: Vec<u8>) -> Mask{
        let mut mask = u64::MAX;
        for bit in b {
            mask ^= 1 << bit
        }
        Mask(mask)
    }

    pub fn one_single(b: u8) -> Mask{
        Mask(0 | (1 << b))
    }

    pub fn zero_single(b: u8) -> Mask {
        Mask(u64::MAX ^ (1 << b))
    }
}
