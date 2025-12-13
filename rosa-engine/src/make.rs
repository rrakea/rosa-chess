//! # Making & Unmaking
//! Instead of copying our position struct on every new move we use the make() and unmake() functions.
//! However this operation is lossy (Castling rights & En passant rights).
//! Since this has to be done multiple times in a row we cant save it in the position struct.
//! Some chess engines use a specialized tables for this information, however Rosa Chess saves it in the 32 bit represenetation
//! of each move. More about that in the move ordering/ move struct.
//! ## Incremental updates to TT Keys
//! Instead of generating the zobrist key new for every operation it is incrementally updated after every operation.

use rosa_lib::mv::*;
use rosa_lib::piece::*;
use rosa_lib::pos::Pos;
use rosa_lib::util;

use crate::mv::*;
use crate::stats;

const BOTTOM_LEFT_SQ: u8 = 0;
const BOTTOM_RIGHT_SQ: u8 = 7;
const TOP_LEFT_SQ: u8 = 56;
const TOP_RIGHT_SQ: u8 = 63;

#[derive(PartialEq, Eq)]
pub enum Legal {
    LEGAL,
    ILLEGAL,
}

pub fn make(p: &mut Pos, mv: &mut Mv, check_legality: bool) -> Legal {
    stats::node_count();
    let color = p.clr;
    let op_color = color.flip();
    let mut castle = p.castle();

    let (start, end) = mv.sq();
    let mut captured_piece_sq = end;
    let mut piece = p.piece_at_sq(start).unwrap_or_else(|| {
        panic!("Error applying mv: {}, to pos: \n{}", mv, p);
    });

    // unset the moving piece
    p.piece_toggle(piece, start);

    let mut ep = None;

    mv.set_old_castle_rights(p.castle());
    match p.ep() {
        Some(file) => {
            mv.set_old_is_ep(true);
            mv.set_old_ep_file(file);
        }
        None => {
            // If the move is the pv move it might still have this set
            mv.set_old_is_ep(false);
            mv.set_old_ep_file(0);
        }
    }

    p.flip_color();

    match mv.flag() {
        Flag::Quiet | Flag::Cap => {}

        Flag::Double => ep = Some(end % 8),

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
            castle.wk = false;
            castle.wq = false;
        }

        Flag::WQC => {
            p.piece_toggle(ClrPiece::WRook, BOTTOM_LEFT_SQ);
            p.piece_toggle(ClrPiece::WRook, BOTTOM_LEFT_SQ + 3);
            castle.wk = false;
            castle.wq = false;
        }

        Flag::BKC => {
            p.piece_toggle(ClrPiece::BRook, TOP_RIGHT_SQ);
            p.piece_toggle(ClrPiece::BRook, TOP_RIGHT_SQ - 2);
            castle.bk = false;
            castle.bq = false;
        }

        Flag::BQC => {
            p.piece_toggle(ClrPiece::BRook, TOP_LEFT_SQ);
            p.piece_toggle(ClrPiece::BRook, TOP_LEFT_SQ + 3);
            castle.bk = false;
            castle.bq = false;
        }
    }

    if mv.is_cap() {
        p.piece_toggle(mv.cap_victim().clr(op_color), captured_piece_sq);
    }

    // This has to be at the end since we need to unset the captured
    // piece first & change the moving piece during a promotion
    p.piece_toggle(piece, end);

    // If: could castle previously && a) Move king, b) moved from rook sq, c) captured rook
    if castle.wk && (piece == ClrPiece::WKing || start == BOTTOM_RIGHT_SQ || end == BOTTOM_RIGHT_SQ)
    {
        castle.wk = false;
    }

    if castle.wq && (piece == ClrPiece::WKing || start == BOTTOM_LEFT_SQ || end == BOTTOM_LEFT_SQ) {
        castle.wq = false;
    }

    if castle.bk && (piece == ClrPiece::BKing || start == TOP_RIGHT_SQ || end == TOP_RIGHT_SQ) {
        castle.bk = false;
    }

    if castle.bq && (piece == ClrPiece::BKing || start == TOP_LEFT_SQ || end == TOP_LEFT_SQ) {
        castle.bq = false;
    }

    p.set_castling(castle);
    p.set_ep(ep);

    // If the king of the moving player is not attacked, the
    // position afterwards is legal
    if check_legality {
        let king_pos = p.piece(Piece::King.clr(color)).get_ones_single();
        if square_attacked(p, color, king_pos) {
            return Legal::ILLEGAL;
        }

        if mv.is_castle() {
            if square_attacked(p, color, start) {
                return Legal::ILLEGAL;
            }

            // Cant be uneven
            let square_after_king = (start as i8 + end as i8) >> 1;
            if square_attacked(p, color, square_after_king as u8) {
                return Legal::ILLEGAL;
            }
        }
    }
    Legal::LEGAL
}

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

    p.set_castling(mv.old_castle_rights());
    if mv.old_is_ep() {
        p.set_ep(Some(mv.old_ep_file()));
    } else {
        p.set_ep(None);
    }
}

pub fn make_null(p: &mut Pos) -> (Legal, Option<u8>) {
    stats::node_count();
    let color = p.clr;
    let king_pos = p.piece(Piece::King.clr(color)).get_ones_single();
    let was_ep = p.ep();
    p.set_ep(None);
    p.flip_color();

    let legal = if square_attacked(p, color, king_pos) {
        Legal::ILLEGAL
    } else {
        Legal::LEGAL
    };

    (legal, was_ep)
}

pub fn unmake_null(p: &mut Pos, was_ep: Option<u8>) {
    p.flip_color();
    p.set_ep(was_ep);
}

/// Basically we pretend there is every possible piece on the square
/// And then & that with the bb of the piece. If non 0 , then the square is attacked by that piece
pub fn square_attacked(p: &Pos, victim_clr: Clr, sq: u8) -> bool {
    let check = |p: &Pos, mask: u64, piece: ClrPiece| mask & p.piece(piece).val() != 0;

    let attacker_color = victim_clr.flip();
    let bishop_mask = magic::bishop_mask(sq, p, true);
    if check(p, bishop_mask, Piece::Bishop.clr(attacker_color))
        || check(p, bishop_mask, Piece::Queen.clr(attacker_color))
    {
        return true;
    }

    let rook_mask = magic::rook_mask(sq, p, true);
    if check(p, rook_mask, Piece::Rook.clr(attacker_color))
        || check(p, rook_mask, Piece::Queen.clr(attacker_color))
    {
        return true;
    }

    let knight_mask = constants::get_mask(Piece::Knight.clr(attacker_color), sq);
    if check(p, knight_mask, Piece::Knight.clr(attacker_color)) {
        return true;
    }

    let (attack_left, attack_right) = if attacker_color.is_white() {
        (sq as i8 - 7, sq as i8 - 9)
    } else {
        (sq as i8 + 7, sq as i8 + 9)
    };

    if (0..64).contains(&attack_left)
        && p.piece_at_sq(attack_left as u8) == Some(Piece::Pawn.clr(attacker_color))
        && util::no_wrap(attack_left as u8, sq)
    {
        return true;
    }

    if (0..64).contains(&attack_right)
        && p.piece_at_sq(attack_right as u8) == Some(Piece::Pawn.clr(attacker_color))
        && util::no_wrap(attack_right as u8, sq)
    {
        return true;
    }

    let king_mask = constants::get_mask(Piece::King.clr(attacker_color), sq);
    if check(p, king_mask, Piece::King.clr(attacker_color)) {
        return true;
    }

    false
}
