use crate::piece::Piece;

/*
    If we encode moves as i32 we can use more flags
    for move ordering purposes

    Move ordering will just use the integer value, so
    the more important flags should be represented as
    large as possible

    At the same time all the unmake flags should be
    represented + as much make stuff we can pack in
    The stuff that we have to save for unmake is:
        - Castling rights (3 bits, maybe 4)
        - Ep (Yes/no: 1 bit, File: 3 bits)
        - Captured piece (3 bits - No king)
        - Was prom

    Ideas:
    - Start + end sq
    - Winning Capture
    - Losing Capture
    - Value in piece square table
    - Check flag (very diffucult to do efficiently with
        our setup)
    -

    0b0000_0000_0000_0000_0000_0000_0000_0000

    >                                 xx xxxx Start sq
    >                          xxxx xx        End sq
    >                      xxx                Ep file
    >                     x                   Is ep
    >                xxxx                     Change castling rights
    >           xxxx                          Prom piece
    >    x xxxx                               Captured piece
    >   x                                     Losing Cap
    >  x                                      Winning cap
    > x                                       Promotion

    -> In this setup we would the top 8 bits for ordering
    -> A cutoff would be trigger when the top 2 bits are flipped
    -> There is quite some redundancie here if we something would be
        more efficient for make/unmake
*/
const START: u32 = 0b_0000_0000_0000_0000_0000_0000_0011_1111;
const END: u32 = 0b_0000_0000_0000_0000_0000_1111_1100_0000;
const EP_FILE: u32 = 0b_0000_0000_0000_0000_0111_0000_0000_0000;
const IS_EP: u32 = 0b_0000_0000_0000_0000_1000_0000_0000_0000;
const CASTLE_RIGHTS: u32 = 0b_0000_0000_0000_1111_0000_0000_0000_0000;
const PROM_PIECE: u32 = 0b_0000_0000_1111_0000_0000_0000_0000_0000;
const CAP_PIECE: u32 = 0b_0001_1111_0000_0000_0000_0000_0000_0000;
const LOSE_CAP: u32 = 0b_0010_0000_0000_0000_0000_0000_0000_0000;
const WIN_CAP: u32 = 0b_0100_0000_0000_0000_0000_0000_0000_0000;
const IS_PROM: u32 = 0b_1000_0000_0000_0000_0000_0000_0000_0000;

const END_OFFSET: u8 = 6;
const EP_FILE_OFFSET: u8 = 12;
const PROM_PIECE_OFFSET: u8 = 20;
const CAP_PIECE_OFFSET: u8 = 24;

const WK_CASTLE: u32 = 0b_0000_0000_0000_0001_0000_0000_0000_0000;
const WQ_CASTLE: u32 = 0b_0000_0000_0000_0010_0000_0000_0000_0000;
const BK_CASTLE: u32 = 0b_0000_0000_0000_0100_0000_0000_0000_0000;
const BQ_CASTLE: u32 = 0b_0000_0000_0000_1000_0000_0000_0000_0000;

const CAP: u32 = 0b_0110_0000_0000_0000_0000_0000_0000_0000;

pub struct LongMv(u32);

impl LongMv {
    pub fn val(&self) -> u32 {
        self.0
    }

    pub fn new(
        start: u8,
        end: u8,
        is_ep: bool,
        ep_file: u8,
        castle_changes: (bool, bool, bool, bool),
        cap_piece: Piece,
        is_prom: bool,
        prom_piece: Piece,
        win_cap: bool,
        lose_cap: bool,
    ) -> LongMv {
        let mut mask = 0;

        mask |= start as u32;
        mask |= (end as u32) << END_OFFSET;
        if is_ep {
            mask |= IS_PROM;
            mask |= (ep_file as u32) << EP_FILE_OFFSET;
        }
        if lose_cap {
            mask |= LOSE_CAP;
        }
        if win_cap {
            mask |= WIN_CAP;
        }
        if win_cap || lose_cap {
            mask |= cap_piece.to_mask() << CAP_PIECE_OFFSET;
        }
        if is_prom {
            mask |= IS_PROM;
            mask |= prom_piece.to_mask() << PROM_PIECE_OFFSET;
        }
        if castle_changes.0 {
            mask |= WK_CASTLE;
        }
        if castle_changes.1 {
            mask |= WQ_CASTLE;
        }
        if castle_changes.2 {
            mask |= BK_CASTLE;
        }
        if castle_changes.3 {
            mask |= BQ_CASTLE;
        }

        LongMv(mask)
    }

    pub fn null() -> LongMv {
        LongMv(0)
    }

    pub fn is_null(&self) -> bool {
        self.start() == self.end()
    }

    pub fn cutoff(&self) -> bool {
        self.is_prom() || self.is_win_cap()
    }

    pub fn is_ep(&self) -> bool {
        self.has(IS_EP)
    }

    pub fn is_cap(&self) -> bool {
        self.has(CAP)
    }

    pub fn is_prom(&self) -> bool {
        self.has(IS_PROM)
    }

    pub fn is_win_cap(&self) -> bool {
        self.has(WIN_CAP)
    }

    pub fn is_lose_cap(&self) -> bool {
        self.has(LOSE_CAP)
    }

    pub fn start(&self) -> u8 {
        (self.val() | START) as u8
    }

    pub fn end(&self) -> u8 {
        ((self.val() | END) >> END_OFFSET) as u8
    }

    pub fn sq(&self) -> (u8, u8) {
        (self.start(), self.end())
    }

    pub fn castle_changes(&self) -> (bool, bool, bool, bool) {
        (
            self.has(WK_CASTLE),
            self.has(WQ_CASTLE),
            self.has(BK_CASTLE),
            self.has(BQ_CASTLE),
        )
    }

    pub fn captured_piece(&self) -> Piece {
        Piece::from_i8(((self.val() | CAP_PIECE) >> CAP_PIECE_OFFSET) as i8)
    }

    pub fn prom_piece(&self) -> Piece {
        Piece::from_i8(((self.val() | PROM_PIECE) >> PROM_PIECE_OFFSET) as i8)
    }

    fn has(&self, mask: u32) -> bool {
        self.val() | mask > 0
    }
}
