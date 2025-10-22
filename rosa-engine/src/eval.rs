use rosa_lib::{piece::ClrPiece, pos};

// Stolen from: https://www.chessprogramming.org/Tapered_Eval
pub fn eval(p: &pos::Pos) -> i32 {
    let mut middelgame = 0;
    let mut endgame = 0;
    let mut phase = STARTPHASE;

    for (sq, piece) in p.piece_iter().enumerate() {
        if let Some(p) = piece {
            let index = p.index();
            middelgame += unsafe { MIDDLEGAME_TABLE[index][sq] };
            endgame += unsafe { ENDGAME_TABLE[index][sq] };
            phase -= PHASEARRAY[index];
        }
    }

    phase = (phase * 256 + (STARTPHASE / 2)) / STARTPHASE;
    (((middelgame * (256 - phase)) + endgame * phase) / 256) * p.clr.as_sign() as i32
}

static mut MIDDLEGAME_TABLE: [[i32; 64]; 12] = [[0; 64]; 12];
static mut ENDGAME_TABLE: [[i32; 64]; 12] = [[0; 64]; 12];

const PHASEARRAY: [i32; 6] = [0, 1, 1, 2, 4, 0];
const STARTPHASE: i32 = 24;

pub fn init_eval() {
    unsafe {
        for sq in 0..64 {
            MIDDLEGAME_TABLE[0][sq] = PAWN_MG + MG_PAWN[sq];
            MIDDLEGAME_TABLE[1][sq] = KNIGHT_MG + MG_KNIGHT[sq];
            MIDDLEGAME_TABLE[2][sq] = BISHOP_MG + MG_BISHOP[sq];
            MIDDLEGAME_TABLE[3][sq] = ROOK_MG + MG_ROOK[sq];
            MIDDLEGAME_TABLE[4][sq] = QUEEN_MG + MG_QUEEN[sq];
            MIDDLEGAME_TABLE[5][sq] = MG_KING[sq];

            MIDDLEGAME_TABLE[6][sq] = -PAWN_MG - MG_PAWN[sq];
            MIDDLEGAME_TABLE[7][sq] = -KNIGHT_MG - MG_KNIGHT[sq];
            MIDDLEGAME_TABLE[8][sq] = -BISHOP_MG - MG_BISHOP[sq];
            MIDDLEGAME_TABLE[9][sq] = -ROOK_MG - MG_ROOK[sq];
            MIDDLEGAME_TABLE[10][sq] = -QUEEN_MG - MG_QUEEN[sq];
            MIDDLEGAME_TABLE[11][sq] = -MG_KING[sq];

            ENDGAME_TABLE[0][sq] = PAWN_EG + EG_PAWN[sq];
            ENDGAME_TABLE[1][sq] = KNIGHT_EG + EG_KNIGHT[sq];
            ENDGAME_TABLE[2][sq] = BISHOP_EG + EG_BISHOP[sq];
            ENDGAME_TABLE[3][sq] = ROOK_EG + EG_ROOK[sq];
            ENDGAME_TABLE[4][sq] = QUEEN_EG + EG_QUEEN[sq];
            ENDGAME_TABLE[5][sq] = EG_KING[sq];

            ENDGAME_TABLE[6][sq] = -PAWN_EG - EG_PAWN[sq];
            ENDGAME_TABLE[7][sq] = -KNIGHT_EG - EG_KNIGHT[sq];
            ENDGAME_TABLE[8][sq] = -BISHOP_EG - EG_BISHOP[sq];
            ENDGAME_TABLE[9][sq] = -ROOK_EG - EG_ROOK[sq];
            ENDGAME_TABLE[10][sq] = -QUEEN_EG - EG_QUEEN[sq];
            ENDGAME_TABLE[11][sq] = -EG_KING[sq];
        }
    }
}

const PAWN_MG: i32 = 82;
const KNIGHT_MG: i32 = 337;
const BISHOP_MG: i32 = 365;
const ROOK_MG: i32 = 477;
const QUEEN_MG: i32 = 1025;

const PAWN_EG: i32 = 94;
const KNIGHT_EG: i32 = 281;
const BISHOP_EG: i32 = 297;
const ROOK_EG: i32 = 512;
const QUEEN_EG: i32 = 936;

const MG_PAWN: [i32; 64] = [0; 64];
const MG_KNIGHT: [i32; 64] = [0; 64];
const MG_BISHOP: [i32; 64] = [0; 64];
const MG_ROOK: [i32; 64] = [0; 64];
const MG_QUEEN: [i32; 64] = [0; 64];
const MG_KING: [i32; 64] = [0; 64];

const EG_PAWN: [i32; 64] = [0; 64];
const EG_KNIGHT: [i32; 64] = [0; 64];
const EG_BISHOP: [i32; 64] = [0; 64];
const EG_ROOK: [i32; 64] = [0; 64];
const EG_QUEEN: [i32; 64] = [0; 64];
const EG_KING: [i32; 64] = [0; 64];
