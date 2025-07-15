/*
Functions for working with moves encoded as u16
These encodings are purely usefull for manipulating the bitboards after words

Encoding inspired by Chess Programming Wiki:
*/
#[repr(u16)]
pub enum MvFlag {
    Quiet = 0,
    Cap = 1,
    WKCastle = 2,
    WQCastle = 3,
    BKCastle = 4,
    BQCastle = 5,
    DoubleP = 6,
    Ep = 7,
    NProm = 8,
    BProm = 9,
    RProm = 10,
    QProm = 11,
    NPromCap = 12,
    BPromCap = 13,
    RPromCap = 14,
    QPromCap = 15,
}

#[derive(Clone, Default)]
pub struct Mv(u16);

impl Mv {
    pub fn new(start: u8, end: u8, flag: MvFlag) -> Mv {
        Mv(start as u16 | (end as u16) << 6 | (flag as u16) << 12)
    }

    pub fn null() -> Mv {
        Mv(0)
    }

    pub fn squares(&self) -> (u8, u8) {
        (self.start(), self.end())
    }

    pub fn start(&self) -> u8 {
        (self.0 & 0b0000_0000_0011_1111) as u8
    }

    pub fn end(&self) -> u8 {
        ((self.0 & 0b0000_1111_1100_0000) >> 6) as u8
    }

    pub fn flag(&self) -> MvFlag {
        unsafe { std::mem::transmute(self.0 >> 12) }
    }

    pub fn is_null(&self) -> bool {
        self.start() == self.end()
    }

    pub fn is_cap(&self) -> bool {
        match self.flag() {
            MvFlag::Cap
            | MvFlag::Ep
            | MvFlag::NPromCap
            | MvFlag::BPromCap
            | MvFlag::RPromCap
            | MvFlag::QPromCap => true,
            _ => false,
        }
    }

    pub fn is_prom(&self) -> bool {
        match self.flag() {
            MvFlag::NProm
            | MvFlag::BProm
            | MvFlag::RProm
            | MvFlag::QProm
            | MvFlag::NPromCap
            | MvFlag::BPromCap
            | MvFlag::RPromCap
            | MvFlag::QPromCap => true,
            _ => false,
        }
    }

    pub fn is_castle(&self) -> bool {
        match self.flag() {
            MvFlag::WKCastle | MvFlag::WQCastle | MvFlag::BKCastle | MvFlag::BQCastle => true,
            _ => false,
        }
    }

    pub fn notation(&self) -> String {
        let (start, end) = self.squares();
        let start = square_name(start);
        let end = square_name(end);       
        start + end.as_str()
    }
}

fn square_name(sq: u8) -> String {
    let file = sq % 8;
    let rank = sq / 8;
    let filestr = (b'a' + file) as char;
    let rankstr = (b'1' + rank) as char;
    format!("{}{}", filestr, rankstr)
}
