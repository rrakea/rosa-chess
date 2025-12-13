//! # Move Generation
//! Since move generation has to be done at every node (except if we find a good TT move/ The null move or TT Move produce a cutoff),
//! it has to be quite optimized. Move generation uses several optimizations techniques, most notably magic bitboards.
//! Since for most nodes we dont actually use most moves, move generation is commonly done in stages
//! (Currently Cap + Non Cap, Possibly Promotions in the future)
//! ## Legal Moves
//! Rosas move generation functions generates pseudo-legal moves i.e. legal moves that dont check if they leave the king in check.
//! The legality is only checked inside of make() using square_not_attacked().
//! Since square_not_attacked() has to check all the oponent moves (with a few optimizations) it is quite expensive
//! ## Magic Bitboards

use super::constants;
use super::magic;

use rosa_lib::board::Board;
use rosa_lib::mv::Mv;
use rosa_lib::piece::*;
use rosa_lib::pos::Pos;
use rosa_lib::util;

use std::collections::BinaryHeap;

pub fn gen_mvs(p: &Pos) -> BinaryHeap<Mv> {
    let mut heap = gen_mvs_stages(p, false);
    heap.append(&mut gen_mvs_stages(p, true));
    heap
}

pub fn gen_mvs_stages(p: &Pos, cap: bool) -> BinaryHeap<Mv> {
    let mut mvs = BinaryHeap::with_capacity(8);
    gen_mv_from_piece(p, &mut mvs, Piece::Knight, cap);
    gen_mv_from_piece(p, &mut mvs, Piece::Bishop, cap);
    gen_mv_from_piece(p, &mut mvs, Piece::Rook, cap);
    gen_mv_from_piece(p, &mut mvs, Piece::Queen, cap);
    gen_mv_from_piece(p, &mut mvs, Piece::King, cap);
    gen_mv_from_piece(p, &mut mvs, Piece::Pawn, cap);
    gen_prom(p, &mut mvs, cap);
    if cap {
        gen_ep(p, &mut mvs);
    } else {
        gen_castle(p, &mut mvs);
        gen_pawn_double(p, &mut mvs);
    }
    mvs
}

fn gen_mv_from_piece(p: &Pos, mvs: &mut BinaryHeap<Mv>, piece: Piece, cap: bool) {
    let piece = piece.clr(p.clr());
    let piece_positions = p.piece(piece).get_ones();
    for sq in piece_positions {
        let possible_moves = get_movemask(p, piece, sq, cap);
        for end_square in possible_moves.get_ones() {
            let victim = p.piece_at_sq(end_square);
            match (cap, victim) {
                (true, Some(v)) => {
                    if v.clr() != piece.clr() {
                        mvs.push(Mv::new_cap(sq, end_square, piece.de_clr(), v.de_clr()));
                    }
                }
                (false, None) => {
                    mvs.push(Mv::new_quiet(sq, end_square, p.clr()));
                }
                _ => (),
            }
        }
    }
}

fn get_movemask(p: &Pos, piece: ClrPiece, sq: u8, cap: bool) -> Board {
    let raw_board = match piece {
        ClrPiece::WKing | ClrPiece::BKing | ClrPiece::WKnight | ClrPiece::BKnight => {
            constants::get_mask(piece, sq)
        }
        ClrPiece::WPawn => {
            constants::get_pawn_mask(Clr::White, sq, cap) & !constants::RANK_MASKS[7]
        }
        ClrPiece::BPawn => {
            constants::get_pawn_mask(Clr::Black, sq, cap) & !constants::RANK_MASKS[0]
        }
        ClrPiece::WRook | ClrPiece::BRook => magic::rook_mask(sq, p, cap),
        ClrPiece::WBishop | ClrPiece::BBishop => magic::bishop_mask(sq, p, cap),
        ClrPiece::WQueen | ClrPiece::BQueen => magic::queen_mask(sq, p, cap),
    };
    Board::new_from(raw_board)
}

fn gen_prom(p: &Pos, mvs: &mut BinaryHeap<Mv>, cap: bool) {
    let rank = if p.clr().is_white() { 6 } else { 1 };
    let pawn_bb = p.piece(Piece::Pawn.clr(p.clr()));
    // Only pawns that are on the last rank
    let relevant_rank = Board::new_from(pawn_bb.val() & constants::RANK_MASKS[rank]);
    for start_sq in relevant_rank.get_ones() {
        let end_quiet = (start_sq as i8 + 8 * p.clr().as_sign()) as u8;
        let cap_right = (start_sq as i8 + 9 * p.clr().as_sign()) as u8;
        let cap_left = (start_sq as i8 + 7 * p.clr().as_sign()) as u8;

        if !cap && p.piece_at_sq(end_quiet).is_none() {
            mvs.extend(Mv::mass_new_prom(start_sq, end_quiet));
        }

        if cap
            && util::no_wrap(start_sq, cap_left)
            && let Some(victim) = p.piece_at_sq(cap_left)
            && victim.clr() != p.clr()
        {
            mvs.extend(Mv::mass_new_prom_cap(start_sq, cap_left, victim.de_clr()));
        }

        if cap
            && util::no_wrap(start_sq, cap_right)
            && let Some(victim) = p.piece_at_sq(cap_right)
            && victim.clr() != p.clr()
        {
            mvs.extend(Mv::mass_new_prom_cap(start_sq, cap_right, victim.de_clr()));
        }
    }
}

fn gen_ep(p: &Pos, mvs: &mut BinaryHeap<Mv>) {
    if !p.is_en_passant() {
        return;
    }

    let file = p.en_passant_file() as i8;
    let left;
    let right;
    let end;
    if p.clr().is_white() {
        left = 4 * 8 + file - 1;
        right = 4 * 8 + file + 1;
        end = 5 * 8 + file;
    } else {
        left = 3 * 8 + file - 1;
        right = 3 * 8 + file + 1;
        end = 2 * 8 + file;
    }

    if (0..64).contains(&left)
        && p.piece_at_sq(left as u8) == Some(Piece::Pawn.clr(p.clr()))
        && util::no_wrap(left as u8, end as u8)
    {
        mvs.push(Mv::new_ep(left as u8, end as u8));
    }

    if (0..64).contains(&right)
        && p.piece_at_sq(right as u8) == Some(Piece::Pawn.clr(p.clr()))
        && util::no_wrap(right as u8, end as u8)
    {
        mvs.push(Mv::new_ep(right as u8, end as u8));
    }
}

fn gen_castle(p: &Pos, mvs: &mut BinaryHeap<Mv>) {
    let can_castle = p.can_castle(p.clr());
    let king_bb = p.piece(Piece::King.clr(p.clr()));
    let king_pos = king_bb.get_ones_single();

    // King side
    // We can skip checking the last square, since that is where the kings ends up
    // -> It is searched again in checking for legal moves
    if can_castle.0
        && p.piece_at_sq(king_pos + 1).is_none()
        && p.piece_at_sq(king_pos + 2).is_none()
    {
        if p.clr().is_white() {
            mvs.push(Mv::new_castle(0));
        } else {
            mvs.push(Mv::new_castle(2));
        };
    }

    // Queen side
    if can_castle.1
        && p.piece_at_sq(king_pos - 1).is_none()
        && p.piece_at_sq(king_pos - 2).is_none()
        && p.piece_at_sq(king_pos - 3).is_none()
    {
        if p.clr().is_white() {
            mvs.push(Mv::new_castle(1));
        } else {
            mvs.push(Mv::new_castle(3));
        };
    }
}

fn gen_pawn_double(p: &Pos, mvs: &mut BinaryHeap<Mv>) {
    let bb = p.piece(Piece::Pawn.clr(p.clr()));
    let rank = if p.clr().is_white() { 1 } else { 6 };

    let second_rank = Board::new_from(bb.val() & constants::RANK_MASKS[rank]);

    for sq in second_rank.get_ones() {
        let one_move = (sq as i8 + (8 * p.clr().as_sign())) as u8;
        let two_move = (sq as i8 + (16 * p.clr().as_sign())) as u8;

        if p.piece_at_sq(one_move).is_none() && p.piece_at_sq(two_move).is_none() {
            mvs.push(Mv::new_double(sq, two_move));
        }
    }
}
