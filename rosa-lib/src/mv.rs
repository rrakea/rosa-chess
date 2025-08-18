use crate::pos::Pos;

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
    >                xxxx                     Changes in castling rights
    >             xx                          Which castle
    >            x                            Is castle
    >           x                             Is double pawn
    >        xx                               Prom piece
    >    x xx                                 Captured piece
    >   x                                     Losing Cap
    >  x                                      Winning cap
    > x                                       Promotion

    -> In this setup we would the top 8 bits for ordering
    -> A cutoff would be trigger when the top 2 bits are flipped
    -> There is quite some redundancie here if we something would be
        more efficient for make/unmake

    Forced: => 20
    Start: 6
    End: 6
    Old castle rights: 4
    Old is ep: 1
    Old ep file: 3

    Mutually Exclusive flags: => 2
    Ep, Prom, Castle, Double -> 2 bits

    Data for flags: => 2
    Ep: 1
    Prom: 2
    Castle: 2
    Double: 0

    Left: Is_cap, cap_piece, cap_prio -> 8 bits left
    Is Cap: 1
    Is win: 1
    Prio upgrade: 3 ( also capped piece)


    On en passant:
        make needs to know if a mv was ep
        to delecte a pawn from the correct sq
        -> set in move gen

        make needs to know if a mv was double
        to set the ep square in pos
        -> set in move gen
        (you could also deduce this potentially)

        unmake needs to know about the pos before

        };
        if it was a double and if yes where to set
        the correct ep in the prev position
        -> need to set this in make

        unmake also needs to know if the move was an ep,
        since you have to shift the reappearing pawn
        -> set in make
*/

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mv(u32);

pub enum Castle {
    WK,
    WQ,
    BK,
    BQ,
}

pub enum Special {
    DOUBLE,
    EP,
    PROM,
    CASTLE,
    NULL,
}

impl Mv {
    pub fn new_def(start: u8, end: u8, is_cap: bool) -> Mv {
        Mv(0)
    }

    pub fn new_prom(start: u8, end: u8, is_cap: bool, piece: i8) -> Mv {
        Mv(0)
    }

    pub fn new_castle(castle_type: Castle) -> Mv {
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
        true
    }

    pub fn sq(&self) -> (u8, u8) {
        (0, 0)
    }
    pub fn is_prom(&self) -> bool {
        true
    }

    pub fn is_cap(&self) -> bool {
        true
    }
    pub fn old_is_ep(&self) -> bool {
        true
    }
    pub fn set_old_is_ep(&self, was: bool) {}
    pub fn is_double(&self) -> bool {
        true
    }
    pub fn is_ep(&self) -> bool {
        true
    }
    pub fn is_castle(&self) -> bool {
        true
    }
    pub fn castle(&self) -> Castle {
        Castle::WK
    }
    pub fn prom_piece(&self) -> i8 {
        0
    }
    pub fn special(&self) -> Special {
        Special::NULL
    }

    pub fn captured_piece(&self) -> i8 {
        0
    }
    pub fn set_captured_piece(&self, piece: i8) {}
    pub fn set_old_castle_rights(&self, rights: (bool, bool, bool, bool)) {}
    pub fn old_castle_rights(&self, color: i8) -> (bool, bool) {
        (true, true)
    }
    pub fn old_ep_file(&self) -> u8 {
        0
    }
    pub fn set_is_ep(&self, is: bool) {}
    pub fn set_old_ep_file(&self, file: u8) {}
    pub fn prittify(&self) -> String {
        String::new()
    }

    pub fn notation(&self) -> String {
        String::new()
    }
}
