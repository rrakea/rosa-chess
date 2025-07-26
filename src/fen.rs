use crate::pos;

const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn starting_pos() -> pos::Pos {
    fen(STARTING_FEN.to_string())
}

// If you pass this a wrong FEN it WILL do bullshit
pub fn fen(fen: String) -> pos::Pos {
    let mut sq: [i8; 64] = [0; 64];
    let split_fen: Vec<&str> = fen.split(" ").collect();

    let ranks = split_fen[0].split("/");
    for (i, rank) in ranks.enumerate() {
        // For some bitchass reason fen goes from rank 8 to rank 1
        let i = 8 - i -1;
        let mut current_sq: usize = 0;
        for piece in rank.chars() {
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

                    _ => scream!("Invalid piece code in FEN: {}", piece),
                };
                sq[current_sq + (i * 8)] = code;
                current_sq += 1;
            }
        }
    }

    let mut active = 1;
    if split_fen[1] == "b" {
        active = -1;
    }

    let mut w_castle = (false, false);
    let mut b_castle = (false, false);

    if split_fen[2] != "-" {
        for castle in split_fen[2].chars() {
            match castle {
                'K' => w_castle.0 = true,
                'Q' => w_castle.1 = true,
                'k' => b_castle.0 = true,
                'q' => b_castle.1 = true,
                _ => scream!("Invalid castle in FEN: {}", castle),
            }
        }
    }

    let mut is_ep = false;
    let mut ep_file = 0;
    if split_fen[3] != "-" {
        is_ep = true;
        let file = split_fen[3].chars().nth(0).unwrap();
        ep_file = file as u8 - b'a';
    }

    pos::Pos::new(sq, active, is_ep, ep_file, w_castle, b_castle)
}
