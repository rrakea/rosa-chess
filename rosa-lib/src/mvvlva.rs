use crate::eval_const;
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
    debug_assert!((0..32).contains(&ret), "Not in range");
    // Saved as 6 bit -> The first bit is flipped
    ret + 32
}

fn index(attacker: Piece, victim: Piece) -> usize {
    debug_assert!(victim != Piece::King, "King cannot be a victim");
    ((victim.val() - 1) * 6 + (attacker.val() - 1)) as usize
}

fn score(attacker: Piece, victim: Piece) -> i32 {
    eval_const::pure_piece_eval(victim) * 10 - eval_const::pure_piece_eval(attacker)
}

pub fn decompress(data: u32) -> (Piece, Piece) {
    unsafe { REVERSE[data as usize - 32] }
}
