use crate::board::Board;
use crate::mv::constants;
use crate::mv::magic;
use crate::mv::mv::{Mv, MvFlag};
use crate::pos;
use crate::pos::Pos;
use crate::util::util;
use std::iter;

// TODO: Castling, EP, Pawn Caps, Pawn Double, Pawn Quiet, Promotions
// TODO: Filter out the best move that is going to appear twice
// This generates pseudo legal moves
// i.e. moves that could leave the king in check
// (It does check if castles are legal)
pub fn gen_mvs(p: &Pos, best: Mv) -> impl Iterator<Item = Mv> {
    iter::once(best)
        .chain(gen_piece_mvs(p, pos::QUEEN * p.active))
        .chain(gen_piece_mvs(p, pos::ROOK * p.active))
        .chain(gen_piece_mvs(p, pos::BISHOP * p.active))
        .chain(gen_piece_mvs(p, pos::KNIGHT * p.active))
        .chain(gen_piece_mvs(p, pos::KING * p.active))
}

// The main function, that does all the work
// It recieves the positions, and the relevant piece.
// This function tries to be as lazy as possible
// i.e. it lazily goes over every square and lazily generates
// all the possible moves from that square
// since this is likely to be broken off early due to pruning
fn gen_piece_mvs(p: &Pos, piece: i8) -> impl Iterator<Item = Mv> {
    let piece_positions = p.piece(piece * p.active).get_ones();
    piece_positions
        .into_iter()
        .flat_map(move |sq| {
            let possible_moves = get_movemask(p, piece, sq).get_ones();
            possible_moves.into_iter().map(move |end_square| {
                let end_sq_piece = p.sq[end_square as usize];
                if end_sq_piece == 0 {
                    Mv::new(sq, end_square, MvFlag::Quiet)
                } else if util::dif_colors(p.active, end_sq_piece) {
                    Mv::new(sq, end_square, MvFlag::Cap)
                } else {
                    Mv::null()
                }
            })
        })
        .filter(|mv| !mv.is_null())
}

// Gets a movemask for the piece and sq
// A Board where all the squares a piece could move from the sq
// are flipped to 1
fn get_movemask(p: &Pos, piece: i8, sq: u8) -> Board {
    let raw_board = match piece {
        pos::KING | pos::BKING | pos::KNIGHT | pos::BKNIGHT | pos::PAWN | pos::BPAWN => {
            constants::get_mask(piece, sq)
        }
        pos::ROOK | pos::BROOK => magic::rook_mask(sq, p),
        pos::BISHOP | pos::BBISHOP => magic::bishop_mask(sq, p),
        pos::QUEEN | pos::BQUEEN => magic::queen_mask(sq, p),
        _ => panic!("Invalid piece in call: {}", piece),
    };
    Board::new(raw_board)
}

fn promotions(p: &Pos) -> impl Iterator<Item = Mv> {
    let rank = if p.active == 1 { 6 } else { 2 };
    let pawn_bb = p.piece(pos::PAWN);
    // Only pawns that are on the last rank
    let relevant_rank = Board::new(pawn_bb.val() & constants::RANK_MASKS[rank]);
    let start_sqs = relevant_rank.get_ones();
    start_sqs.into_iter().flat_map(|start_sq| {
        let end_quiet = (start_sq as i8 + 8 * p.active) as u8;
        let cap_right = (start_sq as i8 + 9 * p.active) as u8;
        let cap_left = (start_sq as i8 + 7 * p.active) as u8;

        let can_quiet = p.sq[(end_quiet) as usize] == 0;
        let can_cap_left = util::no_wrap(start_sq, cap_left)
            && util::dif_colors(p.sq[cap_left as usize], p.sq[start_sq as usize]);
        let can_cap_right = util::no_wrap(start_sq, cap_right)
            && util::dif_colors(p.sq[cap_right as usize], p.sq[start_sq as usize]);

        iter::empty()
            .chain(if can_quiet {
                promotion_helper(start_sq, end_quiet, false)
            } else {
                iter::empty()
            })
            .chain(if can_cap_left {
                promotion_helper(start_sq, cap_left, true)
            } else {
                iter::empty()
            })
            .chain(if can_cap_right {
                promotion_helper(start_sq, cap_right, true)
            } else {
                iter::empty()
            })
    })
}

fn promotion_helper(start: u8, end: u8, is_cap: bool) -> impl Iterator<Item = Mv> {
    let mut mv = Vec::with_capacity(4);
    if is_cap {
        mv.push(Mv::new(start, end, MvFlag::QPromCap));
        mv.push(Mv::new(start, end, MvFlag::RPromCap));
        mv.push(Mv::new(start, end, MvFlag::NPromCap));
        mv.push(Mv::new(start, end, MvFlag::BPromCap));
    } else {
        mv.push(Mv::new(start, end, MvFlag::QProm));
        mv.push(Mv::new(start, end, MvFlag::RProm));
        mv.push(Mv::new(start, end, MvFlag::NProm));
        mv.push(Mv::new(start, end, MvFlag::BProm));
    }
    mv.into_iter()
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
