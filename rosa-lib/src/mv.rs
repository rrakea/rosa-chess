use crate::pos::Pos;
use crate::util;

/*
    Move encoding as u32
    Move ordering is done by value, so the most important
    bits like captured piece and promoted piece should
    be the most significant bits

    We currently have 2 bits left over for buffer

                    Old ep file
                      |
          Prom        |
          Piece       |
      Cap   |         |   Old
     offset |         |  Castle  Start   End
        |   |         |    |       |      |
        |   |         |    |       |      |
      |--| ||         |-| |--| |-----||-----|
    0b0000_0000_0000_0000_0000_0000_0000_0000
             || |--| |
             |   |   |
          Buffer |   |
                 |   |
               Flag  |
                     |
                     |
                    Old
                    is ep
*/

const START: u32 = 0b_0000_0000_0000_0000_0000_1111_1100_0000;
const END: u32 = 0b_0000_0000_0000_0000_0000_0000_0011_1111;
const OLD_CASTLE: u32 = 0b_0000_0000_0000_0000_1111_0000_0000_0000;
const WKC: u32 = 0b_0000_0000_0000_0000_0001_0000_0000_0000;
const WQC: u32 = 0b_0000_0000_0000_0000_0010_0000_0000_0000;
const BKC: u32 = 0b_0000_0000_0000_0000_0100_0000_0000_0000;
const BQC: u32 = 0b_0000_0000_0000_0000_1000_0000_0000_0000;
const OLD_EP_FILE: u32 = 0b_0000_0000_0000_0111_0000_0000_0000_0000;
const OLD_IS_EP: u32 = 0b_0000_0000_0000_1000_0000_0000_0000_0000;
const FLAG: u32 = 0b_0000_0000_1111_0000_0000_0000_0000_0000;
const BUFFER: u32 = 0b_0000_0011_0000_0000_0000_0000_0000_0000;
const PROM_PIECE: u32 = 0b_0000_1100_0000_0000_0000_0000_0000_0000;
const CAP_OFFSET: u32 = 0b_1111_0000_0000_0000_0000_0000_0000_0000;

const START_OFFSET: u32 = 6;
const OLD_EP_FILE_OFFSET: u32 = 16;
const FLAG_OFFSET: u32 = 20;
const PROM_OFFSET: u32 = 26;
const CAP_OFFSET_OFFSET: u32 = 28;

/*
    Capture offset:
    We meassure the capturing piece value and the captured piece
    value to improve move ordering
    -> Not by absulute value but by ranking
    => A pawn capturing a knight is a +1, a rook +2 and a queen +3
    The extreme values are -5 for qxp and +5 pxq
    Q, R, B, N, K, P
    There also needs to be a capture flag, since we cant know
    if it is a pxp or just a quiet move else
*/

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mv(u32);

#[derive(PartialEq)]
#[repr(u8)]
pub enum Flag {
    Quiet = 1,
    Cap = 2,
    Ep = 3,
    Double = 4,
    WKC = 5,
    WQC = 6,
    BKC = 7,
    BQK = 8,
    Prom = 9,
    PromCap = 10,
}

impl Mv {
    pub fn new_quiet(start: u8, end: u8) -> Mv {
        let mut val: u32 = 0;
        val |= end as u32;
        val |= (start as u32) << START_OFFSET;
        Mv(val)
    }

    pub fn new_cap(start: u8, end: u8, capturer: i8, victim: i8) -> Mv {
        let mut mv = Mv::new_quiet(start, end);
        mv.set_flag(Flag::Cap);
        mv
    }

    pub fn new_prom(start: u8, end: u8, is_cap: bool, piece: i8, victim: i8) -> Mv {
        Mv(0)
    }

    pub fn new_castle(castle_type: u8) -> Mv {
        Mv(0)
    }

    pub fn new_ep(start: u8, end: u8) -> Mv {
        Mv(0)
    }
    pub fn new_double(start: u8, end: u8) -> Mv {
        Mv(0)
    }
    pub fn new_from_str(str: &str, p: &Pos) -> Mv {
        Mv(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn sq(&self) -> (u8, u8) {
        ((self.0 & START) as u8, (self.0 & END) as u8)
    }

    pub fn is_prom(&self) -> bool {
        self.flag() == Flag::Prom
    }

    pub fn is_cap(&self) -> bool {
        matches!(self.flag(), Flag::Cap | Flag::PromCap)
    }

    pub fn old_is_ep(&self) -> bool {
        (self.0 | OLD_IS_EP) != 0
    }

    pub fn set_old_is_ep(&mut self) {
        self.0 |= OLD_IS_EP
    }

    pub fn is_ep(&self) -> bool {
        self.flag() == Flag::Ep
    }

    pub fn is_castle(&self) -> bool {
        matches!(self.flag(), Flag::WKC | Flag::BKC | Flag::WQC | Flag::BQK)
    }

    pub fn prom_piece(&self) -> i8 {
        // We safe a knight as 0, as pos::piece its 2 => +2
        ((self.0 | PROM_PIECE) >> PROM_OFFSET) as i8 + 2
    }

    pub fn flag(&self) -> Flag {
        unsafe { std::mem::transmute(((self.0 | FLAG) >> FLAG_OFFSET) as u8) }
    }

    pub fn set_flag(&mut self, flag: Flag) {
        self.0 |= (flag as u32) << FLAG_OFFSET
    }

    pub fn captured_piece(&self, capturer: i8) -> i8 {
        let offset = self.0 >> CAP_OFFSET_OFFSET;
        let capturer = i8::abs(capturer);
    }

    pub fn set_old_castle_rights(&mut self, rights: (bool, bool, bool, bool)) {
        let mut val = 0;
        let (wk, wq, bk, bq) = rights;
        if wk {
            val |= WKC;
        }
        if wq {
            val |= WQC;
        }
        if bk {
            val |= BKC;
        }
        if bq {
            val |= BQC;
        }
        self.0 |= val
    }

    pub fn old_castle_rights(&self, color: i8) -> (bool, bool) {
        if color == 1 {
            (self.0 | WKC > 0, self.0 | WQC > 0)
        } else {
            (self.0 | BKC > 0, self.0 | BQC > 0)
        }
    }

    pub fn old_ep_file(&self) -> u8 {
        (self.0 | OLD_EP_FILE >> OLD_EP_FILE_OFFSET) as u8
    }

    pub fn set_old_ep_file(&mut self, file: u8) {
        self.0 |= (file as u32) << OLD_EP_FILE_OFFSET
    }

    pub fn prittify(&self) -> String {
        String::new()
    }

    pub fn notation(&self) -> String {
        let (start, end) = self.sq();
        let util::file
    }
}
