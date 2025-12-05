//! # Move Representation

use crate::clr::Clr;
use crate::history;
use crate::mvvlva;
use crate::piece::*;
use crate::pos::Pos;
use crate::util;

const START: u32 = 0b_0000_0000_0000_0000_0000_1111_1100_0000;
const END: u32 = 0b_0000_0000_0000_0000_0000_0000_0011_1111;
const WKC: u32 = 0b_0000_0000_0000_0000_0001_0000_0000_0000;
const WQC: u32 = 0b_0000_0000_0000_0000_0010_0000_0000_0000;
const BKC: u32 = 0b_0000_0000_0000_0000_0100_0000_0000_0000;
const BQC: u32 = 0b_0000_0000_0000_0000_1000_0000_0000_0000;
const OLD_EP_FILE: u32 = 0b_0000_0000_0000_0111_0000_0000_0000_0000;
const OLD_IS_EP: u32 = 0b_0000_0000_0000_1000_0000_0000_0000_0000;
const OLD_CASTLE: u32 = 0b_0000_0000_0000_0000_1111_0000_0000_0000;
const FLAG: u32 = 0b_0000_0000_1111_0000_0000_0000_0000_0000;
const PROM_PIECE: u32 = 0b_0000_0011_0000_0000_0000_0000_0000_0000;
const SCORE: u32 = 0b_1111_1100_0000_0000_0000_0000_0000_0000;
const CAP: u32 = 0b_1000_0000_0000_0000_0000_0000_0000_0000;

const START_OFFSET: u32 = 6;
const OLD_EP_FILE_OFFSET: u32 = 16;
const FLAG_OFFSET: u32 = 20;
const PROM_OFFSET: u32 = 24;
const SCORE_OFFSET: u32 = 26;

const WK_STARTING_SQ: u8 = 4;
const BK_STARTING_SQ: u8 = 60;

///  Move encoding as u32  
///  Move ordering is done by value, so the most important  
///  bits like captured piece and promoted piece should  
///  be the most significant bits  
///  The mvvlva could be only 5 bits but there is no other data to store  
///  <pre>
///                  Old ep file  
///                    |  
///         Prom       |  
///         Piece      |  
///           |        |   Old  
///           |        |  Castle  Start   End  
///           |        |    |       |      |  
///           |        |    |       |      |  
///           ||       |-| |--| |-----||-----|  
///  0b0000_0000_0000_0000_0000_0000_0000_0000  
///    |-----|   |--| |  
///      |        |   |  
///      |        |   |  
///    Score      |   |  
///             Flag  |  
///                   |  
///                   |  
///                  Old  
///                  is ep  
///  </pre>
///  The score value is either the mvvlva score for captures  
///  or history heuristic for non captures  
///  We add 32 to the mvvlva score to a) mv order them higher  
///  and b) the very first bit becomes a is_cap() bit  
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mv(u32);

#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum Flag {
    Quiet = 1,
    Cap = 2,
    Ep = 3,
    Double = 4,
    WKC = 5,
    WQC = 6,
    BKC = 7,
    BQC = 8,
    Prom = 9,
    PromCap = 10,
}

impl Mv {
    fn new_mv(start: u8, end: u8) -> Mv {
        debug_assert!((0..64).contains(&start) && (0..64).contains(&end));
        let mut val: u32 = 0;
        val |= end as u32;
        val |= (start as u32) << START_OFFSET;
        Mv(val)
    }

    pub fn new_quiet(start: u8, end: u8, clr: Clr) -> Mv {
        let mut mv = Mv::new_mv(start, end);
        mv.set_flag(Flag::Quiet);
        let score = history::get(&mv, clr);
        mv.set_score(score);
        mv
    }

    pub fn new_cap(start: u8, end: u8, capturer: Piece, victim: Piece) -> Mv {
        let mut mv = Mv::new_mv(start, end);
        mv.set_flag(Flag::Cap);
        let score = mvvlva::compress(capturer, victim);
        mv.set_score(score);
        mv
    }

    pub fn new_prom_cap(start: u8, end: u8, prom_piece: Piece, victim: Piece) -> Mv {
        let mut mv = Mv::new_cap(start, end, Piece::Pawn, victim);
        mv.set_flag(Flag::PromCap);
        mv.0 |= prom_piece.compress_prom() << PROM_OFFSET;
        mv
    }

    pub fn new_prom(start: u8, end: u8, prom_piece: Piece) -> Mv {
        let mut mv = Mv::new_mv(start, end);
        mv.set_flag(Flag::Prom);
        mv.0 |= prom_piece.compress_prom() << PROM_OFFSET;
        mv
    }

    pub fn mass_new_prom(start: u8, end: u8) -> [Mv; 4] {
        [
            Mv::new_prom(start, end, Piece::Knight),
            Mv::new_prom(start, end, Piece::Bishop),
            Mv::new_prom(start, end, Piece::Rook),
            Mv::new_prom(start, end, Piece::Queen),
        ]
    }

    pub fn mass_new_prom_cap(start: u8, end: u8, victim: Piece) -> [Mv; 4] {
        [
            Mv::new_prom_cap(start, end, Piece::Knight, victim),
            Mv::new_prom_cap(start, end, Piece::Bishop, victim),
            Mv::new_prom_cap(start, end, Piece::Rook, victim),
            Mv::new_prom_cap(start, end, Piece::Queen, victim),
        ]
    }

    pub fn new_castle(castle_type: u8) -> Mv {
        match castle_type {
            0 => {
                let mut mv = Mv::new_mv(WK_STARTING_SQ, WK_STARTING_SQ + 2);
                mv.set_flag(Flag::WKC);
                mv
            }
            1 => {
                let mut mv = Mv::new_mv(WK_STARTING_SQ, WK_STARTING_SQ - 2);
                mv.set_flag(Flag::WQC);
                mv
            }
            2 => {
                let mut mv = Mv::new_mv(BK_STARTING_SQ, BK_STARTING_SQ + 2);
                mv.set_flag(Flag::BKC);
                mv
            }
            3 => {
                let mut mv = Mv::new_mv(BK_STARTING_SQ, BK_STARTING_SQ - 2);
                mv.set_flag(Flag::BQC);
                mv
            }
            _ => panic!(),
        }
    }

    pub fn new_ep(start: u8, end: u8) -> Mv {
        let mut mv = Mv::new_cap(start, end, Piece::Pawn, Piece::Pawn);
        mv.set_flag(Flag::Ep);
        mv
    }

    pub fn new_double(start: u8, end: u8) -> Mv {
        let mut mv = Mv::new_mv(start, end);
        mv.set_flag(Flag::Double);
        mv
    }

    pub fn new_from_str(mv_str: &str, p: &Pos) -> Mv {
        let start = util::square_num(&mv_str[..2]);
        let end = util::square_num(&mv_str[2..4]);
        let prom_piece = match &mv_str.chars().nth(4) {
            Some('q') => Some(Piece::Queen),
            Some('b') => Some(Piece::Bishop),
            Some('n') => Some(Piece::Knight),
            Some('r') => Some(Piece::Rook),
            _ => None,
        };

        let piece = p.piece_at_sq(start).unwrap();
        let op_piece = p.piece_at_sq(end);

        let mut mv: Mv;
        match (op_piece, prom_piece) {
            // Prom + Cap
            (Some(op_piece), Some(prom_piece)) => {
                mv = Mv::new_prom_cap(start, end, prom_piece, op_piece.de_clr());
                mv.set_flag(Flag::PromCap);
            }

            // Prom
            (None, Some(prom_piece)) => {
                mv = Mv::new_prom(start, end, prom_piece);
                mv.set_flag(Flag::Prom);
            }

            // Cap
            // Not EP since the pawn is not on the end sq
            (Some(op_piece), None) => {
                mv = Mv::new_cap(start, end, piece.de_clr(), op_piece.de_clr());
                mv.set_flag(Flag::Cap);
            }

            // Quiet Mv
            (None, None) => {
                let mv_diff = start.abs_diff(end);
                mv = if piece.de_clr() == Piece::King && mv_diff == 2 {
                    match (piece.clr(), start < end) {
                        (Clr::White, true) => Mv::new_castle(0),
                        (Clr::White, false) => Mv::new_castle(1),
                        (Clr::Black, true) => Mv::new_castle(2),
                        (Clr::Black, false) => Mv::new_castle(3),
                    }
                } else if piece.de_clr() == Piece::Pawn && mv_diff == 16 {
                    Mv::new_double(start, end)
                } else if piece.de_clr() == Piece::Pawn && (mv_diff == 7 || mv_diff == 9) {
                    Mv::new_ep(start, end)
                } else {
                    Mv::new_quiet(start, end, p.clr)
                }
            }
        }

        mv
    }

    pub const fn null() -> Mv {
        Mv(0)
    }

    pub fn sq(&self) -> (u8, u8) {
        (
            ((self.0 & START) >> START_OFFSET) as u8,
            (self.0 & END) as u8,
        )
    }

    pub fn is_prom(&self) -> bool {
        self.flag() == Flag::Prom
    }

    pub fn is_cap(&self) -> bool {
        self.0 & CAP > 0
    }

    pub fn is_double(&self) -> bool {
        self.flag() == Flag::Double
    }

    pub fn old_is_ep(&self) -> bool {
        (self.0 & OLD_IS_EP) != 0
    }

    pub fn set_old_is_ep(&mut self, val: bool) {
        if val {
            self.0 |= OLD_IS_EP;
        } else {
            self.0 &= !OLD_IS_EP;
        }
    }

    pub fn is_ep(&self) -> bool {
        self.flag() == Flag::Ep
    }

    pub fn is_castle(&self) -> bool {
        matches!(self.flag(), Flag::WKC | Flag::BKC | Flag::WQC | Flag::BQC)
    }

    pub fn prom_piece(&self) -> Piece {
        Piece::decompress_prom((self.0 & PROM_PIECE) >> PROM_OFFSET)
    }

    pub fn flag(&self) -> Flag {
        let val = ((self.0 & FLAG) >> FLAG_OFFSET) as u8;
        debug_assert!(
            (1..11).contains(&val),
            "Mv flag not within bounds, val: {val}, Mv: {:032b}",
            self.0
        );
        unsafe { std::mem::transmute(val) }
    }

    fn set_flag(&mut self, flag: Flag) {
        // Unset the prev flag
        self.0 &= !FLAG;
        self.0 |= (flag as u32) << FLAG_OFFSET
    }

    fn set_score(&mut self, score: u32) {
        debug_assert!(score < 64);
        self.0 |= score << SCORE_OFFSET
    }

    fn capture_data(&self) -> (Piece, Piece) {
        let data = (self.0 & SCORE) >> SCORE_OFFSET;
        mvvlva::decompress(data)
    }

    pub fn cap_capturer(&self) -> Piece {
        self.capture_data().0
    }

    pub fn cap_victim(&self) -> Piece {
        self.capture_data().1
    }

    pub fn set_old_castle_rights(&mut self, rights: (bool, bool, bool, bool)) {
        self.0 &= !OLD_CASTLE;
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

    pub fn old_castle_rights(&self, color: Clr) -> (bool, bool) {
        if color.is_white() {
            (self.0 & WKC > 0, self.0 & WQC > 0)
        } else {
            (self.0 & BKC > 0, self.0 & BQC > 0)
        }
    }

    pub fn old_ep_file(&self) -> u8 {
        ((self.0 & OLD_EP_FILE) >> OLD_EP_FILE_OFFSET) as u8
    }

    pub fn set_old_ep_file(&mut self, file: u8) {
        self.0 &= !OLD_EP_FILE;
        self.0 |= (file as u32) << OLD_EP_FILE_OFFSET
    }
}

// To get it in the uci notation (e.g. e2e4, e7e8q)
impl std::fmt::Display for Mv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (start, end) = self.sq();
        let start = util::square_name(start);
        let end = util::square_name(end);

        let prom_str = if self.is_prom() {
            self.prom_piece().to_string()
        } else {
            String::new()
        };

        write!(f, "{}{}{}", start, end, prom_str)
    }
}

// To get a binary representation
impl std::fmt::Debug for Mv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Mv: {}\nFlag: {:?}\nBin: {:#018b}\nProm Piece: {}\nStart: {}\nEnd: {}",
            self,
            self.flag(),
            self.0,
            self.prom_piece(),
            self.sq().0,
            self.sq().1,
        )
    }
}
