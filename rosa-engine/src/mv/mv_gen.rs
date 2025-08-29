use super::constants;
use super::magic;

use rosa_lib::board::Board;
use rosa_lib::mv::Mv;
use rosa_lib::pos::{self, Pos};
use rosa_lib::util;

use std::collections::BinaryHeap;

pub fn gen_mvs(p: &Pos) -> BinaryHeap<Mv> {
    // 35 is an average amount of moves to expect in a position
    let mut mvs = BinaryHeap::with_capacity(35);
    gen_piece_mvs(p, &mut mvs, pos::QUEEN, true, true);
    gen_piece_mvs(p, &mut mvs, pos::QUEEN, true, true);
    gen_piece_mvs(p, &mut mvs, pos::ROOK, true, true);
    gen_piece_mvs(p, &mut mvs, pos::BISHOP, true, true);
    gen_piece_mvs(p, &mut mvs, pos::KNIGHT, true, true);
    gen_piece_mvs(p, &mut mvs, pos::KING, true, true);
    gen_piece_mvs(p, &mut mvs, pos::PAWN, true, false);
    gen_piece_mvs(p, &mut mvs, pos::PAWN, false, true);
    gen_castle(p, &mut mvs);
    gen_pawn_double(p, &mut mvs);
    gen_ep(p, &mut mvs);
    mvs
}

pub fn gen_piece_mvs(p: &Pos, mvs: &mut BinaryHeap<Mv>, mut piece: i8, can_cap: bool, can_quiet: bool) {
    piece *= p.active;
    let piece_positions = p.piece(piece).get_ones();
    for sq in piece_positions {
        let possible_moves = get_movemask(p, piece, sq, can_cap);
        for end_square in possible_moves.get_ones() {
            let victim = p.piece_at_sq(end_square);
            if can_quiet && victim == 0 {
                mvs.push(Mv::new_quiet(sq, end_square));
            } else if can_cap && victim != 0 && util::dif_colors(p.active, victim) {
                mvs.push(Mv::new_cap(sq, end_square, piece, victim));
            }
        }
    }
}

fn get_movemask(p: &Pos, piece: i8, sq: u8, can_cap: bool) -> Board {
    let raw_board = match piece {
        pos::KING | pos::BKING | pos::KNIGHT | pos::BKNIGHT => constants::get_mask(piece, sq),
        pos::PAWN => constants::get_pawn_mask(1, sq, can_cap) & !constants::RANK_MASKS[7],
        pos::BPAWN => constants::get_pawn_mask(-1, sq, can_cap) & !constants::RANK_MASKS[0],
        pos::ROOK | pos::BROOK => magic::rook_mask(sq, p),
        pos::BISHOP | pos::BBISHOP => magic::bishop_mask(sq, p),
        pos::QUEEN | pos::BQUEEN => magic::queen_mask(sq, p),
        _ => panic!("Invalid piece in call: {}", piece),
    };
    Board::new(raw_board)
}

fn gen_ep(p: &Pos, mvs: &mut BinaryHeap<Mv>) {
    if !p.is_en_passant() {
        return;
    }

    let file = p.en_passant_file() as i8;
    let left;
    let right;
    let end;
    if p.active == 1 {
        left = 4 * 8 + file - 1;
        right = 4 * 8 + file + 1;
        end = 5 * 8 + file;
    } else {
        left = 3 * 8 + file - 1;
        right = 3 * 8 + file + 1;
        end = 2 * 8 + file;
    }

    if (0..64).contains(&left)
        && p.piece_at_sq(left as u8) == pos::PAWN * p.active
        && util::no_wrap(left as u8, end as u8)
    {
        mvs.push(Mv::new_ep(left as u8, end as u8));
    }

    if (0..64).contains(&right)
        && p.piece_at_sq(right as u8) == pos::PAWN * p.active
        && util::no_wrap(right as u8, end as u8)
    {
        mvs.push(Mv::new_ep(right as u8, end as u8));
    }
}

fn gen_castle(p: &Pos, mvs: &mut BinaryHeap<Mv>) {
    let can_castle = p.castling(p.active);
    let king_bb = p.piece(pos::KING * p.active);
    let king_pos = king_bb.get_ones_single();

    // King side
    // We can skip checking the last square, since that is where the kings ends up
    // -> It is searched again in checking for legal moves
    if can_castle.0
        && p.piece_at_sq(king_pos + 1) == 0
        && p.piece_at_sq(king_pos + 2) == 0
        && square_not_attacked(p, king_pos, -p.active)
        && square_not_attacked(p, king_pos + 1, -p.active)
    {
        if p.active == 1 {
            mvs.push(Mv::new_castle(0));
        } else {
            mvs.push(Mv::new_castle(2));
        };
    }

    // Queen side
    if can_castle.1
        && p.piece_at_sq(king_pos - 1) == 0
        && p.piece_at_sq(king_pos - 2) == 0
        && p.piece_at_sq(king_pos - 3) == 0
        && square_not_attacked(p, king_pos, -p.active)
        && square_not_attacked(p, king_pos - 1, -p.active)
    {
        if p.active == 1 {
            mvs.push(Mv::new_castle(1));
        } else {
            mvs.push(Mv::new_castle(3));
        };
    }
}

fn gen_pawn_double(p: &Pos, mvs: &mut BinaryHeap<Mv>) {
    let bb = p.piece(pos::PAWN * p.active);
    let rank = if p.active == 1 { 1 } else { 6 };

    let second_rank = Board::new(bb.val() & constants::RANK_MASKS[rank]);

    for sq in second_rank.get_ones() {
        let one_move = (sq as i8 + (8 * p.active.as_i8())) as u8;
        let two_move = (sq as i8 + (16 * p.active.as_i8())) as u8;

        if p.piece_at_sq(one_move) == 0 && p.piece_at_sq(two_move) == 0 {
            mvs.push(Mv::new_double(sq, two_move));
        }
    }
}

pub fn square_not_attacked(p: &Pos, sq: u8, attacker_color: i8) -> bool {
    // Basically we pretend there is every possible piece on the square
    // And then & that with the bb of the piece. If non 0 , then the square is attacked
    // by that piece
    /*
    let pawn_mask = constants::get_pawn_mask(-attacker_color, sq, true);
    if check_for_piece(p, pawn_mask, pos::PAWN * attacker_color) {
        return false;
    }
    */

    let (attack_left, attack_right) = if attacker_color == 1 {
        (sq as i8 - 7, sq as i8 - 9)
    } else {
        (sq as i8 + 7, sq as i8 + 9)
    };

    if (0..64).contains(&attack_left)
        && p.piece_at_sq(attack_left as u8) == pos::PAWN * attacker_color
        && util::no_wrap(attack_left as u8, sq)
    {
        return false;
    }

    if (0..64).contains(&attack_right)
        && p.piece_at_sq(attack_right as u8) == pos::PAWN * attacker_color
        && util::no_wrap(attack_right as u8, sq)
    {
        return false;
    }

    let king_mask = constants::get_mask(pos::KING * attacker_color, sq);
    if check_for_piece(p, king_mask, pos::KING * attacker_color) {
        return false;
    }

    let knight_mask = constants::get_mask(pos::KNIGHT * attacker_color, sq);
    if check_for_piece(p, knight_mask, pos::KNIGHT * attacker_color) {
        return false;
    }

    let bishop_mask = magic::bishop_mask(sq, p);
    if check_for_piece(p, bishop_mask, pos::BISHOP * attacker_color)
        || check_for_piece(p, bishop_mask, pos::QUEEN * attacker_color)
    {
        return false;
    }

    let rook_mask = magic::rook_mask(sq, p);
    if check_for_piece(p, rook_mask, pos::ROOK * attacker_color)
        || check_for_piece(p, rook_mask, pos::QUEEN * attacker_color)
    {
        return false;
    }

    true
}

fn check_for_piece(p: &pos::Pos, attacker_mask: u64, piece: i8) -> bool {
    let piece_bb = p.piece(piece);
    if attacker_mask & piece_bb.val() != 0 {
        return true;
    }
    false
}
