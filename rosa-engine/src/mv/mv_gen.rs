use super::constants;
use super::magic;

use std::iter;

use rosa_lib::board::Board;
use rosa_lib::mv::*;
use rosa_lib::pos;
use rosa_lib::pos::Pos;
use rosa_lib::util;

// This generates pseudo legal moves
// i.e. moves that could leave the king in check
// (It does check if castles are legal)
// The Moves are ordered roughly in the order of how good they are
// More precise ordering is inside mv_order
// This is more for ordering inside the categories/ ordering the
// large amount of non remarkable moves
// This can contain null moves, which have to be filtered out later
pub fn gen_mvs(p: &Pos) -> impl Iterator<Item = Mv> {
    (promotions(p))
        // Cap Pawn moves
        .chain(gen_piece_mvs(p, pos::PAWN, true, false))
        .chain(gen_ep(p))
        .chain(gen_piece_mvs(p, pos::QUEEN, true, true))
        .chain(gen_piece_mvs(p, pos::ROOK, true, true))
        .chain(gen_piece_mvs(p, pos::BISHOP, true, true))
        .chain(gen_piece_mvs(p, pos::KNIGHT, true, true))
        .chain(gen_castle(p))
        .chain(gen_piece_mvs(p, pos::KING, true, true))
        .chain(gen_pawn_double(p))
        // Queit Pawn moves
        .chain(gen_piece_mvs(p, pos::PAWN, false, true))
}

// The main function, that does all the work
// It recieves the positions, and the relevant piece.
// This function tries to be as lazy as possible
// i.e. it lazily goes over every square and lazily generates
// all the possible moves from that square
// since this is likely to be broken off early due to pruning
fn gen_piece_mvs(
    p: &Pos,
    mut piece: i8,
    can_cap: bool,
    can_quiet: bool,
) -> impl Iterator<Item = Mv> {
    piece *= p.active;
    let piece_positions = p.piece(piece).get_ones();
    piece_positions.into_iter().flat_map(move |sq| {
        let possible_moves = get_movemask(p, piece, sq, can_cap).get_ones();
        possible_moves.into_iter().map(move |end_square| {
            let victim = p.piece_at_sq(end_square);
            if can_quiet && victim == 0 {
                Mv::new_quiet(sq, end_square)
            } else if can_cap && victim != 0 && util::dif_colors(p.active, victim) {
                Mv::new_cap(sq, end_square, piece, victim)
            } else {
                Mv::default()
            }
        })
    })
}

// Gets a movemask for the piece and sq
// A Board where all the squares a piece could move from the sq
// are flipped to 1
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

fn promotions(p: &Pos) -> impl Iterator<Item = Mv> {
    let rank = if p.active == 1 { 6 } else { 1 };
    let pawn_bb = p.piece(pos::PAWN * p.active);
    // Only pawns that are on the last rank
    let relevant_rank = Board::new(pawn_bb.val() & constants::RANK_MASKS[rank]);
    let start_sqs = relevant_rank.get_ones();
    start_sqs.into_iter().flat_map(|start_sq| {
        let end_quiet = (start_sq as i8 + 8 * p.active) as u8;
        let cap_right = (start_sq as i8 + 9 * p.active) as u8;
        let right_victim = p.piece_at_sq(cap_right);
        let cap_left = (start_sq as i8 + 7 * p.active) as u8;
        let left_victim = p.piece_at_sq(cap_left);

        let can_quiet = p.piece_at_sq(end_quiet) == 0;
        let can_cap_left = util::no_wrap(start_sq, cap_left)
            && p.piece_at_sq(cap_left) != 0
            && util::dif_colors(p.piece_at_sq(cap_left), p.piece_at_sq(start_sq));
        let can_cap_right = util::no_wrap(start_sq, cap_right)
            && p.piece_at_sq(cap_right) != 0
            && util::dif_colors(p.piece_at_sq(cap_right), p.piece_at_sq(start_sq));
     
        iter::empty()
            .chain(promotion_helper(start_sq, end_quiet, false, 0, can_quiet))
            .chain(promotion_helper(start_sq, cap_left, true, left_victim, can_cap_left))
            .chain(promotion_helper(start_sq, cap_right, true, right_victim, can_cap_right))
    })
}

fn promotion_helper(start: u8, end: u8, is_cap: bool, victim: i8, legal: bool) -> impl Iterator<Item = Mv> {
    if !legal {
        return Vec::new().into_iter();
    }

    let mut mv = Vec::with_capacity(4);
    if is_cap {
        mv.push(Mv::new_prom(start, end, true, pos::BISHOP, victim));
        mv.push(Mv::new_prom(start, end, true, pos::KNIGHT, victim));
        mv.push(Mv::new_prom(start, end, true, pos::ROOK, victim));
        mv.push(Mv::new_prom(start, end, true, pos::PAWN, victim));
    } else {
        mv.push(Mv::new_prom(start, end, false, pos::BISHOP, victim));
        mv.push(Mv::new_prom(start, end, false, pos::KNIGHT, victim));
        mv.push(Mv::new_prom(start, end, false, pos::ROOK, victim));
        mv.push(Mv::new_prom(start, end, false, pos::PAWN, victim));
    }
    mv.into_iter()
}

fn gen_ep(p: &Pos) -> impl Iterator<Item = Mv> {
    let mut mv = Vec::new();

    if !p.is_en_passant() {
        return mv.into_iter();
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
        mv.push(Mv::new_ep(left as u8, end as u8));
    }

    if (0..64).contains(&right)
        && p.piece_at_sq(right as u8) == pos::PAWN * p.active
        && util::no_wrap(right as u8, end as u8)
    {
        mv.push(Mv::new_ep(right as u8, end as u8));
    }

    mv.into_iter()
}

fn gen_castle(p: &Pos) -> impl Iterator<Item = Mv> {
    let mut mv = Vec::new();

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
            mv.push(Mv::new_castle(Castle::WK));
        } else {
            mv.push(Mv::new_castle(Castle::BK));
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
            mv.push(Mv::new_castle(Castle::WQ));
        } else {
            mv.push(Mv::new_castle(Castle::BQ));
        };
    }
    // Doing this feels wierd, but since we dont really have any for loops
    // etc in this we cant really do it better
    mv.into_iter()
}

// We could almost optimize this, but sadly we have to check whether there is
// a piece in between
fn gen_pawn_double(p: &Pos) -> impl Iterator<Item = Mv> {
    let bb = p.piece(pos::PAWN * p.active);
    let rank = if p.active == 1 { 1 } else { 6 };

    let second_rank = Board::new(bb.val() & constants::RANK_MASKS[rank]);

    second_rank.get_ones().into_iter().map(|sq| {
        let one_move = (sq as i8 + (8 * p.active)) as u8;
        let two_move = (sq as i8 + (16 * p.active)) as u8;

        if p.piece_at_sq(one_move) == 0 && p.piece_at_sq(two_move) == 0 {
            Mv::new_double(sq, two_move)
        } else {
            Mv::default()
        }
    })
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
