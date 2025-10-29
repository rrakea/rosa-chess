use crate::make;

use rosa_lib::clr::Clr;
use rosa_lib::mv::Mv;
use rosa_lib::piece::*;
use rosa_lib::pos;

const START_FEN: [&str; 6] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
    "w",
    "KQkq",
    "-",
    "0",
    "1",
];

pub fn starting_pos(moves: Vec<&str>) -> pos::Pos {
    fen(START_FEN.to_vec(), moves)
}

// If you pass this a wrong FEN it WILL do bullshit
pub fn fen(fen: Vec<&str>, moves: Vec<&str>) -> pos::Pos {
    let mut sq: [ClrPieceOption; 64] = [None; 64];
    let ranks = fen[0].rsplit("/");
    // For some reason fen goes from rank 8 to rank 1
    for (rank, rank_str) in ranks.enumerate() {
        let mut current_sq: usize = 0;
        for piece in rank_str.chars() {
            if piece.is_ascii_digit() {
                current_sq += piece.to_digit(10).unwrap() as usize;
            } else {
                let code = match piece {
                    'P' => ClrPiece::WPawn,
                    'B' => ClrPiece::WBishop,
                    'N' => ClrPiece::WKnight,
                    'R' => ClrPiece::WRook,
                    'Q' => ClrPiece::WQueen,
                    'K' => ClrPiece::WKing,

                    'p' => ClrPiece::BPawn,
                    'b' => ClrPiece::BBishop,
                    'n' => ClrPiece::BKnight,
                    'r' => ClrPiece::BRook,
                    'q' => ClrPiece::BQueen,
                    'k' => ClrPiece::BKing,

                    _ => panic!("Invalid piece code in FEN: {}", piece),
                };
                sq[current_sq + (rank * 8)] = Some(code);
                current_sq += 1;
            }
        }
    }

    let clr = if fen[1] == "b" {
        Clr::Black
    } else if fen[1] == "w" {
        Clr::White
    } else {
        panic!("Invalid color: {}", fen[1]);
    };

    let mut wk = false;
    let mut wq = false;
    let mut bk = false;
    let mut bq = false;

    if fen[2] != "-" {
        for castle in fen[2].chars() {
            match castle {
                'K' => wk = true,
                'Q' => wq = true,
                'k' => bk = true,
                'q' => bq = true,
                _ => panic!("Invalid castle in FEN: {}", castle),
            }
        }
    }

    let mut is_ep = false;
    let mut ep_file = 0;
    if fen[3] != "-" {
        is_ep = true;
        let file = fen[3].chars().next().unwrap();
        ep_file = file as u8 - b'a';
    }

    // split_fen[4] and 5 specify move clocks, which we dont use yet
    let mut pos = pos::Pos::new(sq, clr, is_ep, ep_file, pos::CastleData { wk, wq, bk, bq });

    for mv in moves {
        let mut mv = Mv::new_from_str(mv, &pos);
        let legal = make::make(&mut pos, &mut mv, true);
        if !legal {
            panic!("Move not legal to make")
        }
    }

    pos
}
