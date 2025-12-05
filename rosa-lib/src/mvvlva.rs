//! ## MVVLVA Heuristic
//! MVVLVA stands for most valuable victim, least valuable attacker. This heuristic ranks capture moves
//! based on the assumption that more in general capturing a high value piece is better,
//! and the capturing piece should be at least valuable as possible (Pawn x Queen > Queen x Queen)
//! Since capture chains are not evaluated this can leed to an unoptimal move ordering

use std::collections::HashMap;

use crate::piece::Piece;

static mut VALUES: [u32; 30] = [0; 30];
static mut REVERSE: [(Piece, Piece); 30] = [(Piece::Pawn, Piece::Pawn); 30];

pub fn init_mvvlva() {
    let pieces = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    let mut score_map: HashMap<i32, (Piece, Piece)> = HashMap::new();
    let mut score_arr = Vec::new();

    for att in pieces {
        for vic in pieces {
            if vic == Piece::King {
                continue;
            }
            let score = score(att, vic);
            score_map.insert(score, (att, vic));
            score_arr.push(score);
        }
    }

    score_arr.sort();
    for (i, score) in score_arr.iter().enumerate() {
        let (att, vic) = score_map[score];
        let index = index(att, vic);
        unsafe {
            VALUES[index] = i as u32;
            REVERSE[i] = (att, vic);
        }
    }
}

pub fn compress(attacker: Piece, victim: Piece) -> u32 {
    let index = index(attacker, victim);
    let ret = unsafe { VALUES[index] };
    debug_assert!(
        (0..30).contains(&ret),
        "Not in range, att: {attacker}, victim: {victim}"
    );
    // Saved as 6 bit -> The first bit is flipped
    ret + 32
}

fn index(attacker: Piece, victim: Piece) -> usize {
    debug_assert!(victim != Piece::King, "King cannot be a victim");
    ((victim.val() - 1) * 6 + (attacker.val() - 1)) as usize
}

fn score(attacker: Piece, victim: Piece) -> i32 {
    piece_eval(victim) * 10 - piece_eval(attacker)
}

pub fn decompress(data: u32) -> (Piece, Piece) {
    let index = data as usize - 32;
    unsafe { REVERSE[index] }
}

fn piece_eval(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 341,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 10000,
    }
}
