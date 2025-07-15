use crate::board::Board;
use crate::mv::constants;
use crate::mv::magic;
use crate::mv::mv::{Mv, MvFlag};
use crate::pos;
use crate::pos::Pos;
use crate::util::util;
use std::iter;

/*
    TODO: You can for sure rewrite the pawn moves stuff to also use
    mv_from_movemask()
*/

// Generates an iterator, that lazily generates all the possible moves
// -> When a cutoff is reached the rest of the moves dont get generated at all
// The moves are order such that the most likely good moves are at the top
// e.g. Promotions
pub fn mv_gen(p: &Pos, best: Mv, second: Mv) -> impl Iterator<Item = Mv> {
    iter::once_with(|| wrapper(best, second))
        .chain(iter::once_with(|| promotions(p)))
        .chain(iter::once_with(|| queen(p)))
        .chain(iter::once_with(|| rook(p)))
        .chain(iter::once_with(|| bishop(p)))
        .chain(iter::once_with(|| pawn_cap(p)))
        .chain(iter::once_with(|| knight(p)))
        .chain(iter::once_with(|| castle(p)))
        .chain(iter::once_with(|| king(p)))
        .chain(iter::once_with(|| pawn_ep(p)))
        .chain(iter::once_with(|| pawn_quiet(p)))
        .chain(iter::once_with(|| pawn_double(p)))
        .flat_map(|v| v.into_iter())
}

fn wrapper(best: Mv, second: Mv) -> Vec<Mv> {
    vec![best, second]
}

fn promotions(p: &Pos) -> Vec<Mv> {
    let mut mv = Vec::new();
    let rank = if p.active == 1 { 6 } else { 2 };
    let pawn_bb = p.piece(pos::PAWN);
    // Only pawns that are on the last rank
    let second_rank = pawn_bb.val() & constants::RANK_MASKS[rank];
    if second_rank != 0 {
        let potentials = Board::new(second_rank).get_ones();
        for pawn in potentials {
            // The square is empty
            // Multiply with active since black would be -8 offser
            let second_pos: u8 = (pawn as i8 + 8 * p.active) as u8;
            if p.sq[(second_pos) as usize] == 0 {
                mv.push(Mv::new(pawn, second_pos, MvFlag::NProm));
                mv.push(Mv::new(pawn, second_pos, MvFlag::BProm));
                mv.push(Mv::new(pawn, second_pos, MvFlag::RProm));
                mv.push(Mv::new(pawn, second_pos, MvFlag::QProm));
            }
            let cap_left: u8 = (pawn as i8 + 7 * p.active) as u8;
            if util::no_wrap(pawn, cap_left)
                && util::dif_colors(p.sq[cap_left as usize], p.sq[pawn as usize])
            {
                mv.push(Mv::new(pawn, cap_left, MvFlag::NPromCap));
                mv.push(Mv::new(pawn, cap_left, MvFlag::BPromCap));
                mv.push(Mv::new(pawn, cap_left, MvFlag::RPromCap));
                mv.push(Mv::new(pawn, cap_left, MvFlag::QPromCap));
            }
            let cap_right = (pawn as i8 + 9 * p.active) as u8;
            if util::no_wrap(pawn, cap_right)
                && util::dif_colors(p.sq[cap_right as usize], p.sq[pawn as usize])
            {
                mv.push(Mv::new(pawn, cap_right, MvFlag::NPromCap));
                mv.push(Mv::new(pawn, cap_right, MvFlag::BPromCap));
                mv.push(Mv::new(pawn, cap_right, MvFlag::RPromCap));
                mv.push(Mv::new(pawn, cap_right, MvFlag::QPromCap));
            }
        }
    }
    mv
}

fn queen(p: &Pos) -> Vec<Mv> {
    let bb = p.piece(pos::QUEEN * p.active);
    let squares = bb.get_ones();
    let mut mv = Vec::new();
    for sq in squares {
        let movemask = magic::queen_mask(sq, p);
        mv.append(&mut mv_from_movemask(p, movemask, sq));
    }
    mv
}

fn rook(p: &Pos) -> Vec<Mv> {
    let bb = p.piece(pos::ROOK * p.active);
    let squares = bb.get_ones();
    let mut mv = Vec::new();
    for sq in squares {
        let movemask = magic::rook_mask(sq, p);
        mv.append(&mut mv_from_movemask(p, movemask, sq));
    }
    mv
}

fn bishop(p: &Pos) -> Vec<Mv> {
    let bb = p.piece(pos::BISHOP * p.active);
    let squares = bb.get_ones();
    let mut mv = Vec::new();
    for sq in squares {
        let movemask = magic::bishop_mask(sq, p);
        mv.append(&mut mv_from_movemask(p, movemask, sq));
    }
    mv
}

fn king(p: &Pos) -> Vec<Mv> {
    let bb = p.piece(pos::KING * p.active);
    // There can only be one king
    let sq = bb.get_ones_single();
    let mut mv = Vec::new();
    let movemask = unsafe { constants::KING_MASKS[sq as usize] };
    mv.append(&mut mv_from_movemask(p, movemask, sq));
    mv
}

fn knight(p: &Pos) -> Vec<Mv> {
    let bb = p.piece(pos::KNIGHT * p.active);
    let squares = bb.get_ones();
    let mut mv = Vec::new();
    for sq in squares {
        let movemask = unsafe { constants::KNIGHT_MASKS[sq as usize] };
        mv.append(&mut mv_from_movemask(p, movemask, sq));
    }
    mv
}

// Gets a mask of all the possible moves a piece can move from
// its current square -> checks whether the squares are occupied by
// enemy/ owner pieces and generates the proper u16 representation
fn mv_from_movemask(p: &Pos, move_mask: u64, start: u8) -> Vec<Mv> {
    let pos_moves = Board::new(move_mask).get_ones();
    let mut mv = Vec::new();
    for pos_mv in pos_moves {
        let end_sq_val = p.sq[pos_mv as usize];
        if end_sq_val == 0 {
            mv.push(Mv::new(start, pos_mv, MvFlag::Quiet));
        } else if util::dif_colors(end_sq_val, p.active) {
            mv.push(Mv::new(start, pos_mv, MvFlag::Cap));
        } // You dont do anything if the piece is the same color as you
    }
    mv
}

fn pawn_ep(p: &Pos) -> Vec<Mv> {
    let mut mv = Vec::new();
    if p.is_en_passant() {
        let ep_file = p.en_passant_file() as i8;
        let left: i8 = ep_file - 1;
        let right: i8 = ep_file + 1;
        let rank = if p.active == 1 { 5 } else { 4 };
        let pawn_code = if p.active == 1 { pos::PAWN } else { -pos::PAWN };
        if left != -1 && p.sq[(rank * 8 + left) as usize] == pawn_code {
            mv.push(Mv::new(
                (rank * 8 + left) as u8,
                (rank * 8 + ep_file) as u8,
                MvFlag::Ep,
            ));
        }

        if right != 8 && p.sq[(rank * 8 + right) as usize] == pawn_code {
            mv.push(Mv::new(
                (rank * 8 + right) as u8,
                (rank * 8 + ep_file) as u8,
                MvFlag::Ep,
            ));
        }
    }

    mv
}

fn castle(p: &Pos) -> Vec<Mv> {
    let mut mv = Vec::new();

    let can_castle = p.castling(p.active);
    let king_bb = p.piece(pos::KING);
    let king_pos = king_bb.get_ones_single();

    // King side
    if can_castle.0
        && p.sq[king_pos as usize + 1] == 0
        && p.sq[king_pos as usize + 2] == 0
        && square_attacked(p, king_pos, -p.active)
        && square_attacked(p, king_pos + 1, -p.active)
        && square_attacked(p, king_pos + 2, -p.active)
    {
        let code = if p.active == 1 {
            MvFlag::WKCastle
        } else {
            MvFlag::BKCastle
        };
        mv.push(Mv::new(king_pos, king_pos + 2, code))
    }

    // Queen side
    if can_castle.1
        && p.sq[king_pos as usize - 1] == 0
        && p.sq[king_pos as usize - 2] == 0
        && p.sq[king_pos as usize - 3] == 0
        && square_attacked(p, king_pos, -p.active)
        && square_attacked(p, king_pos - 1, -p.active)
        && square_attacked(p, king_pos - 2, -p.active)
    {
        let code = if p.active == 1 {
            MvFlag::WQCastle
        } else {
            MvFlag::BQCastle
        };
        mv.push(Mv::new(king_pos, king_pos - 2, code));
    }
    mv
}

fn pawn_quiet(p: &Pos) -> Vec<Mv> {
    let mut mv = Vec::new();

    let bb = p.piece(pos::PAWN * p.active);
    let offset = if p.active == 1 { 8 } else { -8 };
    for pawn in bb.get_ones() {
        let second_pos = (pawn as i8 + offset) as u8;
        // Pawns not on promotion squares
        if second_pos > 8 && second_pos < (7 * 8) && p.sq[second_pos as usize] == 0 {
            mv.push(Mv::new(pawn, second_pos, MvFlag::Quiet));
        }
    }
    mv
}

fn pawn_double(p: &Pos) -> Vec<Mv> {
    let mut mv = Vec::new();

    let bb = p.piece(pos::PAWN * p.active);
    let rank = if p.active == 1 { 2 } else { 6 };

    let second_rank = Board::new(bb.val() ^ constants::RANK_MASKS[rank]);

    if second_rank.empty() {
        return mv;
    }

    for pawn in second_rank.get_ones() {
        let one_move = pawn as i8 + 8 * p.active;
        let two_move = pawn as i8 + 16 * p.active;

        if p.sq[one_move as usize] == 0 && p.sq[two_move as usize] == 0 {
            mv.push(Mv::new(pawn, two_move as u8, MvFlag::DoubleP));
        }
    }

    mv
}

fn pawn_cap(p: &Pos) -> Vec<Mv> {
    Vec::new()
}

pub fn square_attacked(p: &Pos, sq: u8, attacked_by: i8) -> bool {
    // Basically we pretend there is every possible piece on the square
    // And then & that with the bb of the piece. If non 0 , then the square is attacked
    // by that piece
    let pawn_mask = constants::get_mask(pos::PAWN * attacked_by, sq);
    if check_for_piece(p, pawn_mask, pos::PAWN * attacked_by) {
        return false;
    }

    let king_mask = constants::get_mask(pos::KING * attacked_by, sq);
    if check_for_piece(p, king_mask, pos::KING * attacked_by) {
        return false;
    }

    let knight_mask = constants::get_mask(pos::KNIGHT, sq);
    if check_for_piece(p, knight_mask, pos::KNIGHT * attacked_by) {
        return false;
    }

    let bishop_mask = magic::bishop_mask(sq, p);
    if check_for_piece(p, bishop_mask, pos::BISHOP * attacked_by) {
        return false;
    }

    let rook_mask = magic::rook_mask(sq, p);
    if check_for_piece(p, rook_mask, pos::ROOK * attacked_by) {
        return false;
    }

    let queen_mask = rook_mask | bishop_mask;
    if check_for_piece(p, queen_mask, pos::QUEEN * attacked_by) {
        return false;
    }

    return true;
}

fn check_for_piece(p: &pos::Pos, attacker_mask: u64, piece: i8) -> bool {
    let piece_bb = p.piece(piece);
    if attacker_mask & piece_bb.val() != 0 {
        return true;
    }
    false
}
