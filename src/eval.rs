use crate::eval_const;
use crate::pos;

// Since we are using negamax this evaluation function has to return
// a value relative to the side to move

pub fn eval(p: &pos::Pos) -> i32 {
    let mut eval = 0;

    // This way of checking for endgame is inspired by this entry:
    // https://www.chessprogramming.org/Simplified_Evaluation_Function
    // Not really the best way, tempered eval would be way better
    let w_minor_piece_count =
        p.piece(pos::BISHOP).count() + p.piece(pos::KNIGHT).count() + p.piece(pos::ROOK).count();
    let w_endgame = p.piece(pos::QUEEN).empty() || w_minor_piece_count <= 1;

    let b_minor_piece_count =
        p.piece(pos::BBISHOP).count() + p.piece(pos::BKNIGHT).count() + p.piece(pos::BROOK).count();
    let b_endgame = p.piece(pos::BQUEEN).empty() || b_minor_piece_count <= 1;

    let endgame = w_endgame && b_endgame;

    for piece in pos::PIECE_VAL_ARRAY {
        for sq in p.piece(piece).get_ones() {
            // Same color
            if piece * p.active > 0 {
                eval += eval_const::piece_eval(sq, piece, p.active, endgame);
            } else {
                eval -= eval_const::piece_eval(sq, piece, p.active, endgame);            }
        }
    }

    eval * p.active as i32
}
