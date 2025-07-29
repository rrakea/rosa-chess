use super::mv_gen;
use crate::mv::mv::{Mv, MvFlag};
use crate::pos;
use crate::pos::Pos;
use crate::table::Key;
use crate::util;

const BOTTOM_LEFT_SQ: u8 = 0;
const BOTTOM_RIGHT_SQ: u8 = 7;
const TOP_LEFT_SQ: u8 = 56;
const TOP_RIGHT_SQ: u8 = 63;

// This function takes a position and a move
// and returns the position after the move
// It update the zobrist key, the bitboards,
// the square based board and the attack boards.
pub fn apply(old_p: &Pos, mv: &Mv, old_key: &mut Key) -> Option<(Pos, Key)> {
    debug!("Applying mv: {}", mv.prittify());
    if mv.is_null() {
        debug!("Null move apply!");
        return None;
    }

    let mut npos = old_p.clone();
    let mut nkey = *old_key;

    let mut w_castle = old_p.castling(1);
    let mut b_castle = old_p.castling(-1);
    let mut ep_file = old_p.en_passant_file();
    let mut is_ep = false;

    // Remove the old ep file from the hash
    if old_p.is_en_passant() {
        nkey.en_passant(ep_file);
        ep_file = 0;
    }

    let old_act = npos.active;
    let new_act = -npos.active;

    npos.active = new_act;
    nkey.color();
    let sq = mv.squares();
    let start = sq.0;
    let end = sq.1;

    // Unset the castling rights since its easier to unset them once
    // and then later set them again rather than update them everywhere they could change
    if w_castle.0 {
        nkey.castle(1, true);
    }
    if w_castle.1 {
        nkey.castle(1, false)
    }
    if b_castle.0 {
        nkey.castle(-1, true)
    }
    if b_castle.1 {
        nkey.castle(-1, false)
    }

    let piece = npos.sq[start as usize];
    let op_piece = npos.sq[end as usize];

    if piece == 0 {
        debug!("Piece is 0, mv: {}", mv.prittify());
        println!("Old board: ");
        old_p.print();
        println!("New board: ");
        npos.print();
    }
    if op_piece == 0 && mv.is_cap() {
        debug!("OpPiece is 0, mv: {}", mv.prittify());
    }

    // Set the values in the square based represantation
    npos.sq[start as usize] = 0;
    npos.sq[end as usize] = piece;
    nkey.piece(start, piece);
    if op_piece != 0 {
        nkey.piece(end, piece);
    }

    // Sets the bitboard for the moved piece
    npos.piece_mut(piece).unset(start);
    npos.piece_mut(piece).set(end);

    if mv.is_cap() {
        debug!("Capture: Piece is: {}, in pos: {}", op_piece, npos.sq[end as usize]);
        npos.piece_mut(op_piece).unset(end);
    }

    match mv.flag() {
        // The capture is set bellow together with promotion captures
        MvFlag::Quiet | MvFlag::Cap | MvFlag::Ep => (),
        MvFlag::DoubleP => {
            ep_file = util::file(end);
            is_ep = true;
            nkey.en_passant(ep_file);
        }

        MvFlag::BProm | MvFlag::BPromCap => {
            let piece = pos::BISHOP * old_act;
            npos.sq[end as usize] = piece;
            nkey.piece(end, piece);
        }
        MvFlag::NProm | MvFlag::NPromCap => {
            let piece = if old_act == 1 {
                pos::KNIGHT
            } else {
                pos::BKNIGHT
            };
            npos.sq[end as usize] = piece;
            nkey.piece(end, piece);
        }
        MvFlag::RProm | MvFlag::RPromCap => {
            let piece = if old_act == 1 { pos::ROOK } else { pos::BROOK };
            npos.sq[end as usize] = piece;
            nkey.piece(end, piece);
        }
        MvFlag::QProm | MvFlag::QPromCap => {
            let piece = if old_act == 1 {
                pos::QUEEN
            } else {
                pos::BQUEEN
            };
            npos.sq[end as usize] = piece;
            nkey.piece(end, piece);
        }

        MvFlag::WKCastle => {
            npos.piece_mut(pos::ROOK).unset(BOTTOM_RIGHT_SQ);
            npos.sq[BOTTOM_RIGHT_SQ as usize] = 0;
            nkey.piece(BOTTOM_RIGHT_SQ, pos::ROOK);

            npos.piece_mut(pos::ROOK).set(BOTTOM_RIGHT_SQ - 2);
            npos.sq[BOTTOM_RIGHT_SQ as usize - 2] = pos::ROOK;
            nkey.piece(BOTTOM_RIGHT_SQ - 2, pos::ROOK);

            w_castle = (false, false);
        }
        MvFlag::WQCastle => {
            npos.piece_mut(pos::ROOK).set(BOTTOM_LEFT_SQ);
            npos.sq[BOTTOM_LEFT_SQ as usize] = 0;
            nkey.piece(BOTTOM_LEFT_SQ, pos::ROOK);

            npos.piece_mut(pos::ROOK).set(BOTTOM_LEFT_SQ + 3);
            npos.sq[BOTTOM_LEFT_SQ as usize + 3] = pos::ROOK;
            nkey.piece(BOTTOM_LEFT_SQ + 3, pos::ROOK);

            w_castle = (false, false);
        }
        MvFlag::BKCastle => {
            npos.sq[TOP_RIGHT_SQ as usize] = 0;
            nkey.piece(TOP_RIGHT_SQ, pos::BROOK);
            npos.piece_mut(pos::BROOK).set(TOP_RIGHT_SQ - 2);
            npos.sq[TOP_RIGHT_SQ as usize - 2] = pos::BROOK;
            nkey.piece(TOP_RIGHT_SQ - 2, pos::BROOK);

            b_castle = (false, false);
        }
        MvFlag::BQCastle => {
            npos.piece_mut(pos::BROOK).set(TOP_LEFT_SQ);
            npos.sq[TOP_LEFT_SQ as usize] = 0;
            nkey.piece(TOP_LEFT_SQ, pos::BROOK);

            npos.piece_mut(pos::BROOK).set(TOP_LEFT_SQ + 3);
            npos.sq[TOP_LEFT_SQ as usize + 3] = pos::BROOK;
            nkey.piece(TOP_LEFT_SQ + 3, pos::BROOK);

            b_castle = (false, false);
        }
    }
    // This is active player agnostic
    if w_castle.0 {
        // If the king moves || the rook moved from starting square || the rook if captured
        if piece == pos::KING || (piece == pos::ROOK && start == 0) || end == 0 {
            w_castle.0 = false;
        } else {
            //Castling is still legal
            nkey.castle(old_act, true);
        }
    }

    if w_castle.1 {
        if piece == pos::KING || (piece == pos::ROOK && start == 7) || end == 7 {
            w_castle.1 = false;
        } else {
            nkey.castle(old_act, false);
        }
    }

    if b_castle.0 {
        if piece == pos::BKING || (piece == pos::BROOK && start == 63) || end == 63 {
            b_castle.0 = false;
        } else {
            nkey.castle(old_act, true);
        }
    }

    if b_castle.1 {
        if piece == pos::BKING || (piece == pos::BROOK && start == 56) || end == 56 {
            b_castle.1 = false;
        } else {
            nkey.castle(old_act, false);
        }
    }

    pos::gen_full(&mut npos);
    npos.data = pos::gen_data(is_ep, ep_file, w_castle, b_castle);

    /*
    if is_legal(&npos) {
        return Some(npos);
    } else {
        return None;
    }
    */
    Some((npos, nkey))
}

fn is_legal(p: &Pos) -> bool {
    let king_pos = p.piece(pos::KING * -p.active).get_ones_single();
    debug!("Checking legality with king at pos: {}", king_pos);
    mv_gen::square_attacked(p, king_pos, -p.active)
}
