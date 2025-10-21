use rosa_lib::clr::Clr;
use rosa_lib::mv::*;
use rosa_lib::piece::*;
use rosa_lib::pos::{self, Pos};
use rosa_lib::util;

use crate::mv::mv_gen;

const BOTTOM_LEFT_SQ: u8 = 0;
const BOTTOM_RIGHT_SQ: u8 = 7;
const TOP_LEFT_SQ: u8 = 56;
const TOP_RIGHT_SQ: u8 = 63;

pub fn make(p: &mut Pos, mv: &mut Mv) -> bool {
    let color = p.clr;
    let op_color = color.flip();

    let (start, end) = mv.sq();
    let mut captured_piece_sq = end;
    let mut piece = p.piece_at_sq(start).unwrap_or_else(|| {
        println!("{:?}", mv);
        println!("Pos: \n{}", p);
        panic!();
    });

    // unset the moving piece
    p.piece_toggle(piece, start);

    let (mut wk, mut wq) = p.can_castle(Clr::White);
    let (mut bk, mut bq) = p.can_castle(Clr::Black);

    let mut is_ep = false;
    let mut ep_file = 0;

    mv.set_old_castle_rights((wk, wq, bk, bq));
    if p.is_en_passant() {
        mv.set_old_is_ep(true);
        mv.set_old_ep_file(p.en_passant_file());
    }

    p.flip_color();

    match mv.flag() {
        Flag::Quiet | Flag::Cap => {}

        Flag::Double => {
            is_ep = true;
            ep_file = util::file(end);
        }

        Flag::Ep => {
            captured_piece_sq = match color {
                Clr::White => end - 8,
                Clr::Black => end + 8,
            }
        }

        Flag::Prom | Flag::PromCap => {
            piece = mv.prom_piece().clr(color);
        }

        Flag::WKC => {
            p.piece_toggle(ClrPiece::WRook, BOTTOM_RIGHT_SQ);
            p.piece_toggle(ClrPiece::WRook, BOTTOM_RIGHT_SQ - 2);
            wk = false;
            wq = false;
        }

        Flag::WQC => {
            p.piece_toggle(ClrPiece::WRook, BOTTOM_LEFT_SQ);
            p.piece_toggle(ClrPiece::WRook, BOTTOM_LEFT_SQ + 3);
            wk = false;
            wq = false;
        }

        Flag::BKC => {
            p.piece_toggle(ClrPiece::BRook, TOP_RIGHT_SQ);
            p.piece_toggle(ClrPiece::BRook, TOP_RIGHT_SQ - 2);
            bk = false;
            bq = false;
        }

        Flag::BQC => {
            p.piece_toggle(ClrPiece::BRook, TOP_LEFT_SQ);
            p.piece_toggle(ClrPiece::BRook, TOP_LEFT_SQ + 3);
            bk = false;
            bq = false;
        }
    }

    if mv.is_cap() {
        p.piece_toggle(mv.cap_victim().clr(op_color), captured_piece_sq);
    }

    // This has to be at the end since we need to unset the captured
    // piece first & change the moving piece during a promotion
    p.piece_toggle(piece, end);

    // If: could castle previously && a) Move king, b) moved from rook sq, c) captured rook
    if wk && (piece == ClrPiece::WKing || start == BOTTOM_RIGHT_SQ || end == BOTTOM_RIGHT_SQ) {
        wk = false;
    }

    if wq && (piece == ClrPiece::WKing || start == BOTTOM_LEFT_SQ || end == BOTTOM_LEFT_SQ) {
        wq = false;
    }

    if bk && (piece == ClrPiece::BKing || start == TOP_RIGHT_SQ || end == TOP_RIGHT_SQ) {
        bk = false;
    }

    if bq && (piece == ClrPiece::BKing || start == TOP_LEFT_SQ || end == TOP_LEFT_SQ) {
        bq = false;
    }

    p.gen_new_data(is_ep, ep_file, pos::CastleData { wk, wq, bk, bq });

    // If the king of the moving player is not attacked, the
    // position afterwards is legal
    let king_pos = p.piece(Piece::King.clr(color)).get_ones_single();
    mv_gen::square_not_attacked(p, king_pos, color.flip())
}

#[inline(always)]
pub fn unmake(p: &mut Pos, mv: &mut Mv) {
    let color = p.clr.flip();
    let op_color = p.clr;

    let (start, end) = mv.sq();
    let mut captured_piece_sq = end;
    let mut piece = p.piece_at_sq(end).unwrap();

    p.flip_color();
    p.piece_toggle(piece, end);

    match mv.flag() {
        Flag::Quiet | Flag::Cap | Flag::Double => {}

        Flag::Ep => {
            captured_piece_sq = match color {
                Clr::White => end - 8,
                Clr::Black => end + 8,
            };
        }

        Flag::Prom | Flag::PromCap => piece = Piece::Pawn.clr(color),

        Flag::WKC => {
            p.piece_toggle(ClrPiece::WRook, BOTTOM_RIGHT_SQ);
            p.piece_toggle(ClrPiece::WRook, BOTTOM_RIGHT_SQ - 2);
        }

        Flag::WQC => {
            p.piece_toggle(ClrPiece::WRook, BOTTOM_LEFT_SQ);
            p.piece_toggle(ClrPiece::WRook, BOTTOM_LEFT_SQ + 3);
        }

        Flag::BKC => {
            p.piece_toggle(ClrPiece::BRook, TOP_RIGHT_SQ);
            p.piece_toggle(ClrPiece::BRook, TOP_RIGHT_SQ - 2);
        }

        Flag::BQC => {
            p.piece_toggle(ClrPiece::BRook, TOP_LEFT_SQ);
            p.piece_toggle(ClrPiece::BRook, TOP_LEFT_SQ + 3);
        }
    }

    p.piece_toggle(piece, start);

    if mv.is_cap() {
        p.piece_toggle(mv.cap_victim().clr(op_color), captured_piece_sq);
    }


    let (wk, wq) = mv.old_castle_rights(Clr::White);
    let (bk, bq) = mv.old_castle_rights(Clr::Black);

    p.gen_new_data(mv.old_is_ep(), mv.old_ep_file(), pos::CastleData { wk, wq, bk, bq });
}
