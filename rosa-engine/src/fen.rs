use crate::make;
use rosa_lib::mv::Mv;
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
    let mut sq: [i8; 64] = [0; 64];
    let ranks = fen[0].rsplit("/");
    // For some reason fen goes from rank 8 to rank 1
    for (rank, rank_str) in ranks.enumerate() {
        let mut current_sq: usize = 0;
        for piece in rank_str.chars() {
            if piece.is_ascii_digit() {
                current_sq += piece.to_digit(10).unwrap() as usize;
            } else {
                let code = match piece {
                    'P' => pos::PAWN,
                    'B' => pos::BISHOP,
                    'N' => pos::KNIGHT,
                    'R' => pos::ROOK,
                    'Q' => pos::QUEEN,
                    'K' => pos::KING,

                    'p' => pos::BPAWN,
                    'b' => pos::BBISHOP,
                    'n' => pos::BKNIGHT,
                    'r' => pos::BROOK,
                    'q' => pos::BQUEEN,
                    'k' => pos::BKING,

                    _ => panic!("Invalid piece code in FEN: {}", piece),
                };
                sq[current_sq + (rank * 8)] = code;
                current_sq += 1;
            }
        }
    }

    let mut active = 1;
    if fen[1] == "b" {
        active = -1;
    }

    let mut w_castle = (false, false);
    let mut b_castle = (false, false);

    if fen[2] != "-" {
        for castle in fen[2].chars() {
            match castle {
                'K' => w_castle.0 = true,
                'Q' => w_castle.1 = true,
                'k' => b_castle.0 = true,
                'q' => b_castle.1 = true,
                _ => panic!("Invalid castle in FEN: {}", castle),
            }
        }
    }

    let mut is_ep = false;
    let mut ep_file = 0;
    if fen[3] != "-" {
        is_ep = true;
        let file = fen[3].chars().nth(0).unwrap();
        ep_file = file as u8 - b'a';
    }

    // split_fen[4] and 5 specify move clocks, which we dont use yet
    let mut pos = pos::Pos::new(sq, active, is_ep, ep_file, w_castle, b_castle);

    for mv in moves {
        let mut mv = Mv::new_from_str(mv, &pos);
        let res = make::make(&mut pos, &mut mv, true);
        if res {
            panic!("Move not legal to make")
        }
    }

    pos
}
