use crate::pos;
use crate::util;
/*

Functions for working with moves encoded as u16
These encodings are purely usefull for manipulating the bitboards after words

Encoding inspired by Chess Programming Wiki:
*/
#[repr(u16)]
#[derive(Debug, PartialEq)]
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

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Mv(u16);

impl Mv {
    pub fn new(start: u8, end: u8, flag: MvFlag) -> Mv {
        Mv(start as u16 | (end as u16) << 6 | (flag as u16) << 12)
    }

    pub fn from_str(mv_str: &str, p: &pos::Pos) -> Mv {
        println!("Castling and en passant not implemented yet :(");
        let start = util::square_num(&mv_str[..2]);
        let end = util::square_num(&mv_str[2..4]);
        let mut flag = MvFlag::Quiet;
        let piece = p.piece_at_sq(start);
        let op_piece = p.piece_at_sq(end);
        let mv_diff = (end - start) as i8 * p.active;

        if op_piece != 0 {
            flag = MvFlag::Cap;
        }

        if piece == pos::PAWN * p.active && (util::rank(end) == 0 || util::rank(end) == 7) {
            let prom_piece = mv_str
                .chars()
                .nth(4)
                .expect("Promotion does not specify piece");
            if op_piece != 0 {
                flag = match prom_piece {
                    'q' => MvFlag::QPromCap,
                    'n' => MvFlag::NPromCap,
                    'b' => MvFlag::BPromCap,
                    'r' => MvFlag::RPromCap,
                    _ => scream!("Promotion piece not valid"),
                };
            } else {
                flag = match prom_piece {
                    'q' => MvFlag::QProm,
                    'n' => MvFlag::NProm,
                    'b' => MvFlag::BProm,
                    'r' => MvFlag::RProm,
                    _ => scream!("Promotion piece not valid"),
                };
            }
        }

        if piece == pos::PAWN * p.active {
            if mv_diff == 16 {
                flag = MvFlag::DoubleP;
            } else if op_piece == 0 && (mv_diff == 7 || mv_diff == 9){
                flag = MvFlag::Ep;
            }
        }

        if piece == pos::KING * p.active {
            match (p.active, mv_diff) {
                (1, 2) => flag = MvFlag::WKCastle,
                (1, -2) => flag = MvFlag::WQCastle,
                (-1, 2) => flag = MvFlag::BKCastle,
                (-1, -2) => flag = MvFlag::BQCastle,
                _ => ()
            };
        }

        Mv::new(start, end, flag)
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

    pub fn is_ep(&self) -> bool {
        self.flag() == MvFlag::Ep
    }

    pub fn notation(&self) -> String {
        let (start, end) = self.squares();
        let start = util::square_name(start);
        let end = util::square_name(end);

        let mut prom_str = "";
        if self.is_prom() {
            prom_str = match self.flag() {
                MvFlag::QProm | MvFlag::QPromCap => "q",
                MvFlag::RProm | MvFlag::RPromCap => "r",
                MvFlag::BProm | MvFlag::BPromCap => "b",
                MvFlag::NProm | MvFlag::NPromCap => "n",
                _ => "",
            }
        }
        start + end.as_str() + prom_str
    }

    pub fn prittify(&self) -> String {
        format!("{}, {:?}", self.notation(), self.flag())
    }
}
