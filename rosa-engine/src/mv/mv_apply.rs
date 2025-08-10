use super::mv_gen;

use rosa_lib::mv::*;
use rosa_lib::pos::Pos;
use rosa_lib::pos;
use rosa_lib::util;

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

    let color = pos.active;
    pos.flip_color();

    let (start, end) = mv.squares();

    let mut op_end = end;
    if mv.is_ep() {
        op_end = if color == 1 { end - 8 } else { end + 8 };
    }

    let piece = pos.piece_at_sq(start);
    let op_piece = pos.piece_at_sq(op_end);

    if piece == 0 {
        println!("Piece is 0, mv: {}", mv.prittify());
    }
    if op_piece == 0 && mv.is_cap() {
        println!("OpPiece is 0, mv: {}", mv.prittify());
    }

    pos.piece_toggle(piece, start);
    pos.piece_toggle(op_piece, op_end);
    // We dont set the pawn board  at a promotion, since the piece changes
    if !mv.is_prom() {
        pos.piece_toggle(piece, end);
    }

    match mv.flag() {
        MvFlag::Quiet | MvFlag::Cap | MvFlag::Ep => (),
        MvFlag::DoubleP => {
            ep_file = util::file(end);
            is_ep = true;
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
            pos.piece_toggle(pos::BROOK, TOP_RIGHT_SQ);
            pos.piece_toggle(pos::BROOK, TOP_RIGHT_SQ - 2);
        }
        MvFlag::BQCastle => {
            pos.piece_toggle(pos::BROOK, TOP_LEFT_SQ);
            pos.piece_toggle(pos::BROOK, TOP_LEFT_SQ + 3);
        }
    }

    if piece == pos::KING {
        wk_castle = false;
        wq_castle = false;
    }
    if piece == pos::BKING {
        bk_castle = false;
        bq_castle = false;
    }

    // Could castle previously && The rook moved from starting square || the rook if captured
    if wk_castle && ((piece == pos::ROOK && start == 7) || end == 7) {
        wk_castle = false;
    }

    if wq_castle && ((piece == pos::ROOK && start == 0) || end == 0) {
        wq_castle = false;
    }

    if bk_castle && ((piece == pos::BROOK && start == 63) || end == 63) {
        bk_castle = false;
    }

    if bq_castle && ((piece == pos::BROOK && start == 56) || end == 56) {
        bq_castle = false;
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
    mv_gen::square_not_attacked(p, king_pos, p.active)
}
