use super::mv_gen;

use crate::mv::mv::{Mv, MvFlag};
use crate::pos;
use crate::pos::Pos;
use crate::util;

const BOTTOM_LEFT_SQ: u8 = 0;
const BOTTOM_RIGHT_SQ: u8 = 7;
const TOP_LEFT_SQ: u8 = 56;
const TOP_RIGHT_SQ: u8 = 63;

// This function takes a position and a move
// and returns the position after the move
// It update the zobrist key, the bitboards,
// the square based board and the attack boards.
pub fn apply(old_p: &Pos, mv: &Mv) -> Option<Pos> {
    let mut pos = old_p.clone();

    let (mut wk_castle, mut wq_castle) = pos.castling(1);
    let (mut bk_castle, mut bq_castle) = pos.castling(-1);
    let mut ep_file = pos.en_passant_file();
    let mut is_ep = false;

    // Remove the old ep file from the hash
    if old_p.is_en_passant() {
        pos.key.en_passant(ep_file);
        ep_file = 0;
    }

    let color = pos.active;
    pos.active = -color;
    pos.key.color();
    let (start, end) = mv.squares();

    // Unset the castling rights since its easier to unset them once
    // and then later set them again rather than update them everywhere they could change
    if wk_castle {
        pos.key.castle(1, true);
    }
    if wq_castle {
        pos.key.castle(1, false)
    }
    if bk_castle {
        pos.key.castle(-1, true)
    }
    if bq_castle {
        pos.key.castle(-1, false)
    }

    let piece = pos.piece_at_sq(start);
    let mut op_piece = pos.piece_at_sq(end);
    if mv.is_ep() {
        let ep_sq = if pos.active == 1 { end + 8 } else { end - 8 };
        op_piece = pos.piece_at_sq(ep_sq);
    }

    if piece == 0 {
        debug!("Piece is 0, mv: {}", mv.prittify());
    }
    if op_piece == 0 && mv.is_cap() {
        debug!("OpPiece is 0, mv: {}", mv.prittify());
    }

    pos.piece_toggle(piece, start);
    pos.piece_toggle(op_piece, end);
    // We dont set the pawn board  at a promotion, since the piece changes
    if !mv.is_prom() {
        pos.piece_toggle(piece, end);
    }

    match mv.flag() {
        MvFlag::Quiet | MvFlag::Cap | MvFlag::Ep => (),
        MvFlag::DoubleP => {
            ep_file = util::file(end);
            is_ep = true;
            pos.key.en_passant(ep_file);
        }

        MvFlag::BProm | MvFlag::BPromCap => {
            pos.piece_toggle(pos::BISHOP * color, end);
        }
        MvFlag::NProm | MvFlag::NPromCap => {
            pos.piece_toggle(pos::KNIGHT * color, end);
        }
        MvFlag::RProm | MvFlag::RPromCap => {
            pos.piece_toggle(pos::ROOK * color, end);
        }
        MvFlag::QProm | MvFlag::QPromCap => {
            pos.piece_toggle(pos::QUEEN * color, end);
        }

        // For all the casteling, we dont need to set the king, since
        // castles are encoded as the king moving 2 squares
        MvFlag::WKCastle => {
            pos.piece_toggle(pos::ROOK, BOTTOM_RIGHT_SQ);
            pos.piece_toggle(pos::ROOK, BOTTOM_RIGHT_SQ - 2);
        }
        MvFlag::WQCastle => {
            pos.piece_toggle(pos::ROOK, BOTTOM_LEFT_SQ);
            pos.piece_toggle(pos::ROOK, BOTTOM_LEFT_SQ + 3);
        }
        MvFlag::BKCastle => {
            pos.piece_toggle(pos::ROOK, TOP_RIGHT_SQ);
            pos.piece_toggle(pos::ROOK, TOP_RIGHT_SQ - 2);
        }
        MvFlag::BQCastle => {
            pos.piece_toggle(pos::ROOK, TOP_LEFT_SQ);
            pos.piece_toggle(pos::ROOK, TOP_LEFT_SQ + 3);
        }
    }

    if piece == pos::KING {
        wk_castle = false;
        wq_castle = false;
        bk_castle = false;
        bq_castle = false;
    }

    // This is active player agnostic
    if wk_castle {
        // The rook moved from starting square || the rook if captured
        if (piece == pos::ROOK && start == 0) || end == 0 {
            wk_castle = false;
        } else {
            //Castling is still legal
            pos.key.castle(color, true);
        }
    }

    if wq_castle {
        if (piece == pos::ROOK && start == 7) || end == 7 {
            wq_castle = false;
        } else {
            pos.key.castle(color, false);
        }
    }

    if bk_castle {
        if (piece == pos::BROOK && start == 63) || end == 63 {
            bk_castle = false;
        } else {
            pos.key.castle(color, true);
        }
    }

    if bq_castle {
        if (piece == pos::BROOK && start == 56) || end == 56 {
            bq_castle = false;
        } else {
            pos.key.castle(color, false);
        }
    }

    pos.gen_new_data(
        is_ep,
        ep_file,
        (wk_castle, wq_castle),
        (bk_castle, bq_castle),
    );
    pos.gen_new_full();

    if is_legal(&pos) {
        Some(pos)
    } else {
        None
    }
}

fn is_legal(p: &Pos) -> bool {
    let king_pos = p.piece(pos::KING * -p.active).get_ones_single();
    mv_gen::square_attacked(p, king_pos, p.active)
}
