use rosa_lib::mv::*;
use rosa_lib::pos::{self, Pos};
use rosa_lib::util;

use crate::mv::mv_gen;

const BOTTOM_LEFT_SQ: u8 = 0;
const BOTTOM_RIGHT_SQ: u8 = 7;
const TOP_LEFT_SQ: u8 = 56;
const TOP_RIGHT_SQ: u8 = 63;

pub fn make(p: &mut Pos, mv: &mut Mv, make: bool) -> bool {
    let color = p.active;
    let (start, mut end) = mv.sq();
    let piece = p.piece_at_sq(start);

    // unset the moving piece
    p.piece_toggle(piece, start);

    // Promotions are the only move where the moving piece
    // does not "move" to end (changes piece)
    if !mv.is_prom() {
        p.piece_toggle(piece, end);
    }

    let (mut wk_castle, mut wq_castle) = if make {
        p.castling(1)
    } else {
        mv.old_castle_rights(1)
    };
    let (mut bk_castle, mut bq_castle) = if make {
        p.castling(-1)
    } else {
        mv.old_castle_rights(-1)
    };

    let mut is_ep = false;
    let mut ep_file = 0;

    p.flip_color();
    if make {
        mv.set_old_castle_rights((wk_castle, wq_castle, bk_castle, bq_castle));
        if p.is_en_passant() {
            mv.set_old_is_ep();
            mv.set_old_ep_file(p.en_passant_file());
        }
    }

    match mv.flag() {
        Flag::Double => {
            if make {
                is_ep = true;
                ep_file = util::file(end);
            } else {
                is_ep = mv.old_is_ep();
                ep_file = mv.old_ep_file();
            }
        }

        Flag::Ep => {
            end = match color {
                1 => end - 8,
                -1 => end + 8,
                _ => end,
            }
        }

        Flag::Prom => {
            let prom_piece = mv.prom_piece();
            p.piece_toggle(prom_piece, end);
        }

        Flag::PromCap => {
            match mv.castle() {
                Castle::WK => {
                    p.piece_toggle(pos::ROOK, BOTTOM_RIGHT_SQ);
                    p.piece_toggle(pos::ROOK, BOTTOM_RIGHT_SQ - 2);
                }
                Castle::WQ => {
                    p.piece_toggle(pos::ROOK, BOTTOM_LEFT_SQ);
                    p.piece_toggle(pos::ROOK, BOTTOM_LEFT_SQ + 3);
                }
                Castle::BK => {
                    p.piece_toggle(pos::BROOK, TOP_RIGHT_SQ);
                    p.piece_toggle(pos::BROOK, TOP_RIGHT_SQ - 2);
                }
                Castle::BQ => {
                    p.piece_toggle(pos::BROOK, TOP_LEFT_SQ);
                    p.piece_toggle(pos::BROOK, TOP_LEFT_SQ + 3);
                }
            }

            if make {
                match color {
                    1 => {
                        wk_castle = false;
                        wq_castle = false
                    }
                    -1 => {
                        bk_castle = false;
                        bq_castle = false;
                    }
                    _ => (),
                }
            }
        }

        _ => (),
    }

    // cap after special since you need to move the end with ep
    if mv.is_cap() {
        p.piece_toggle(mv.captured_piece(piece), end);
    }

    if make {
        // If: could castle previously && a) Move king, b) moved from rook sq, c) captured rook
        if wk_castle && (piece == pos::KING || start == BOTTOM_RIGHT_SQ || end == BOTTOM_RIGHT_SQ) {
            wk_castle = false;
        }

        if wq_castle && (piece == pos::KING || start == BOTTOM_LEFT_SQ || end == BOTTOM_LEFT_SQ) {
            wq_castle = false;
        }

        if bk_castle && (piece == pos::BKING || start == TOP_RIGHT_SQ || end == TOP_RIGHT_SQ) {
            bk_castle = false;
        }

        if bq_castle && (piece == pos::BKING || start == TOP_LEFT_SQ || end == TOP_LEFT_SQ) {
            bq_castle = false;
        }
    }

    // Rememver changes for unmake
    p.gen_new_data(is_ep, ep_file, wk_castle, wq_castle, bk_castle, bq_castle);

    if make {
        // If the king of the moving player is not attacked, the
        // position afterwards is legal
        let king_pos = p.piece(pos::KING * color).get_ones_single();
        mv_gen::square_not_attacked(p, king_pos, -color)
    } else {
        true
    }
}

/*
pub fn unmake(p: &mut Pos, mv: &Mv) {
    let color = p.active;
    p.flip_color();

    let (start, mut end) = mv.sq();
    let piece = p.piece_at_sq(start);

    p.piece_toggle(piece, start);

    if !mv.is_prom() {
        p.piece_toggle(piece, end);
    }

    match mv.special() {
        SpecialMoveFlag::EP => {
            end = match color {
                1 => end - 8,
                -1 => end + 8,
                _ => end,
            }
        }
        SpecialMoveFlag::PROM => {
            p.piece_toggle(mv.prom_piece(), end);
        }
        SpecialMoveFlag::CASTLE => match mv.castle() {
            CastleType::WK => {
                p.piece_toggle(pos::ROOK, BOTTOM_RIGHT_SQ);
                p.piece_toggle(pos::ROOK, BOTTOM_RIGHT_SQ - 2);
            }
            CastleType::WQ => {
                p.piece_toggle(pos::ROOK, BOTTOM_LEFT_SQ);
                p.piece_toggle(pos::ROOK, BOTTOM_LEFT_SQ + 3);
            }
            CastleType::BK => {
                p.piece_toggle(pos::BROOK, TOP_RIGHT_SQ);
                p.piece_toggle(pos::BROOK, TOP_RIGHT_SQ - 2);
            }
            CastleType::BQ => {
                p.piece_toggle(pos::BROOK, TOP_LEFT_SQ);
                p.piece_toggle(pos::BROOK, TOP_LEFT_SQ + 3);
            }
        },
        _ => (),
    }

    if mv.is_cap() {
        p.piece_toggle(mv.captured_piece(), end);
    }

    let (wk_castle, wq_castle, bk_castle, bq_castle) = mv.old_castle_rights();
    p.gen_new_data(
        mv.old_is_ep(),
        mv.old_ep_file(),
        wk_castle,
        wq_castle,
        bk_castle,
        bq_castle,
    );
    p.gen_new_full();
}
*/
