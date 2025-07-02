use crate::mv::mv::{Mv, MvFlag};
use crate::pos::bboard;
use crate::pos::pos;
use crate::pos::pos::Pos;
use crate::table::table;
use crate::util;

const BOTTOM_LEFT_SQ: usize = 0;
const BOTTOM_RIGHT_SQ: usize = 7;
const TOP_LEFT_SQ: usize = 56;
const TOP_RIGHT_SQ: usize = 63;

pub fn apply(p: &Pos, mv: &Mv) -> Option<Pos> {
    let mut npos = p.clone();

    // This is the new zobrist key
    // It will be update through this functions through calls to table::xxx_hash()
    let mut key = p.key;

    let mut w_castle = p.castling(1);
    let mut b_castle = p.castling(-1);
    let mut ep_file = p.en_passant_file();
    // Remove the old ep file from the hash
    key ^= table::ep_hash(ep_file);

    let old_act = npos.active;
    let new_act = -npos.active;
    npos.active = new_act;
    key ^= table::color_hash();
    let sq = mv.squares();
    let start = sq.0 as usize;
    let end = sq.1 as usize;

    // Unset the castling rights since its easier to unset them once
    // and then later set them again rather than update them everywhere they could change
    if w_castle.0 {
        key ^= table::castel_hash(old_act, true)
    }
    if w_castle.1 {
        key ^= table::castel_hash(old_act, false)
    }
    if b_castle.0 {
        key ^= table::castel_hash(old_act, true)
    }
    if b_castle.1 {
        key ^= table::castel_hash(old_act, false)
    }

    // Set the values in the square base represantation
    let piece = npos.sq[start as usize];
    let op_piece = npos.sq[end as usize];
    npos.sq[start] = 0;
    npos.sq[end] = piece;
    key ^= table::piece_hash(start, piece);
    if op_piece != 0 {
        key ^= table::piece_hash(end, piece);
    }

    // Sets the bitboard for the moved piece
    set_bboard(&mut npos, piece, start, 0);
    set_bboard(&mut npos, piece, end, 1);

    match mv.flag() {
        // The capture is set bellow together with promotion captures
        MvFlag::Quiet | MvFlag::Cap | MvFlag::Ep => (),
        MvFlag::DoubleP => {
            ep_file = util::util::file(end as u8);
            key ^= table::ep_hash(ep_file);
        }

        MvFlag::BProm | MvFlag::BPromCap => {
            let piece = if old_act == 1 {
                pos::WBISHOP
            } else {
                pos::BBISHOP
            };
            npos.sq[end] = piece;
            key ^= table::piece_hash(end, piece);
        }
        MvFlag::NProm | MvFlag::NPromCap => {
            let piece = if old_act == 1 {
                pos::WKNIGHT
            } else {
                pos::BKNIGHT
            };
            npos.sq[end] = piece;
            key ^= table::piece_hash(end, piece);
        }
        MvFlag::RProm | MvFlag::RPromCap => {
            let piece = if old_act == 1 { pos::WROOK } else { pos::BROOK };
            npos.sq[end] = piece;
            key ^= table::piece_hash(end, piece);
        }
        MvFlag::QProm | MvFlag::QPromCap => {
            let piece = if old_act == 1 {
                pos::WQUEEN
            } else {
                pos::BQUEEN
            };
            npos.sq[end] = piece;
            key ^= table::piece_hash(end, piece);
        }

        MvFlag::WKCastle => {
            set_bboard(&mut npos, pos::WROOK, BOTTOM_RIGHT_SQ, 0);
            npos.sq[BOTTOM_RIGHT_SQ] = 0;
            key ^= table::piece_hash(BOTTOM_RIGHT_SQ, pos::WROOK);

            set_bboard(&mut npos, pos::WROOK, BOTTOM_RIGHT_SQ - 2, 1);
            npos.sq[BOTTOM_RIGHT_SQ - 2] = pos::WROOK;
            key ^= table::piece_hash(BOTTOM_RIGHT_SQ - 2, pos::WROOK);

            w_castle = (false, false);
        }
        MvFlag::WQCastle => {
            set_bboard(&mut npos, pos::WROOK, BOTTOM_LEFT_SQ, 1);
            npos.sq[BOTTOM_LEFT_SQ] = 0;
            key ^= table::piece_hash(BOTTOM_LEFT_SQ, pos::WROOK);

            set_bboard(&mut npos, pos::WROOK, BOTTOM_LEFT_SQ + 3, 1);
            npos.sq[BOTTOM_LEFT_SQ + 3] = pos::WROOK;
            key ^= table::piece_hash(BOTTOM_LEFT_SQ + 3, pos::WROOK);

            w_castle = (false, false);
        }
        MvFlag::BKCastle => {
            npos.sq[TOP_RIGHT_SQ] = 0;
            key ^= table::piece_hash(TOP_RIGHT_SQ, pos::BROOK);

            set_bboard(&mut npos, pos::BROOK, TOP_RIGHT_SQ - 2, 1);
            npos.sq[TOP_RIGHT_SQ - 2] = pos::BROOK;
            key ^= table::piece_hash(TOP_RIGHT_SQ - 2, pos::BROOK);

            b_castle = (false, false);
        }
        MvFlag::BQCastle => {
            set_bboard(&mut npos, pos::BROOK, TOP_LEFT_SQ, 1);
            npos.sq[TOP_LEFT_SQ] = 0;
            key ^= table::piece_hash(TOP_LEFT_SQ, pos::BROOK);

            set_bboard(&mut npos, pos::BROOK, TOP_LEFT_SQ + 3, 1);
            npos.sq[TOP_LEFT_SQ + 3] = pos::BROOK;
            key ^= table::piece_hash(TOP_LEFT_SQ + 3, pos::BROOK);

            b_castle = (false, false);
        }
    }

    // This sets the of the captured piece in the bitboard to 0
    if mv.is_cap() {
        set_bboard(&mut npos, op_piece, end, 0);
    }

    // This is active player agnostic
    if w_castle.0 {
        // If the king moves || the rook moved from starting square || the rook if captured
        if piece == pos::WKING || (piece == pos::WROOK && start == 0) || end == 0 {
            w_castle.0 = false;
        } else {
            //Castling is still legal
            key ^= table::castel_hash(old_act, true);
        }
    }

    if w_castle.1 {
        if piece == pos::WKING || (piece == pos::WROOK && start == 7) || end == 7 {
            w_castle.1 = false;
        } else {
            key ^= table::castel_hash(old_act, false);
        }
    }

    if b_castle.0 {
        if piece == pos::BKING || (piece == pos::BROOK && start == 63) || end == 63 {
            b_castle.0 = false;
        } else {
            key ^= table::castel_hash(old_act, true);
        }
    }

    if b_castle.1 {
        if piece == pos::BKING || (piece == pos::BROOK && start == 56) || end == 56 {
            b_castle.1 = false;
        } else {
            key ^= table::castel_hash(old_act, false);
        }
    }

    npos.full = bboard::bb_all(p);
    npos.data = pos::gen_data(ep_file, w_castle, b_castle);
    npos.key = key;

    if legal_pos(&npos) {
        return Some(npos);
    } else {
        return None;
    }
}

fn set_bboard(p: &mut Pos, piece: i8, sq: usize, val: i8) {
    let mut bb = match piece {
        pos::WPAWN => p.wp,
        pos::WBISHOP => p.wb,
        pos::WKNIGHT => p.wn,
        pos::WROOK => p.wr,
        pos::WQUEEN => p.wq,
        pos::WKING => p.wk,
        pos::BPAWN => p.bp,
        pos::BBISHOP => p.bb,
        pos::BKNIGHT => p.bn,
        pos::BROOK => p.br,
        pos::BQUEEN => p.bq,
        pos::BKING => p.bk,
        _ => panic!("Invalid piece code: {}", piece),
    };

    if val == 0 {
        bb &= !(1 << sq);
    } else {
        bb &= 1 << sq;
    }

    match piece {
        pos::WPAWN => p.wp = bb,
        pos::WBISHOP => p.wb = bb,
        pos::WKNIGHT => p.wn = bb,
        pos::WROOK => p.wr = bb,
        pos::WQUEEN => p.wq = bb,
        pos::WKING => p.wk = bb,
        pos::BPAWN => p.bp = bb,
        pos::BBISHOP => p.bb = bb,
        _ => panic!("Invalid piece code: {}", piece)
    }
}

pub fn legal_pos(p: &Pos) -> bool {
    // This is turned around since the opponents king is allowed to be in check
    // Since the switching of the active player has already happend
    // The only illegal thing would be, when the active player could capture the king
    let king_bb = if p.active == 1 { p.bk } else { p.wk };
    let attack_bb = if p.active == 1 { p.wattack } else { p.battack };
    king_bb & attack_bb != 0
}
