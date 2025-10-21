use rosa_engine::*;
use rosa_lib::pos;

use rosa_engine::runtime::init;
use rosa_engine::debug_search::*;

const DEBUG_SEARCH: bool = false;

fn start_search(p: &mut pos::Pos, expected: [u64; 6]) {
    if !DEBUG_SEARCH {
        for (i, res) in expected.iter().enumerate() {
            println!("Starting Depth: {i}");
            let count = counting_search(p, i as u8);
            println!("Depth: {i}, Count: {count}");
            assert_eq!(count, *res);
        }
    } else {
        debug_search(p, 6, &mut Vec::new());
    }
}

#[test]
fn starting_pos() {
    init();
    let mut pos = fen::starting_pos(Vec::new());
    let expected = [1, 20, 400, 8902, 197281, 4865609];
    start_search(&mut pos, expected);
}

#[test]
fn tricky_pos_1() {
    init();
    let mut pos = fen::fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "
            .split_ascii_whitespace()
            .collect(),
        Vec::new(),
    );
    let expected = [1, 48, 2039, 97862, 4085603, 193690690];
    start_search(&mut pos, expected);
}

#[test]
fn tricky_pos_2() {
    init();
    let mut pos = fen::fen(
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 "
            .split_ascii_whitespace()
            .collect(),
        Vec::new(),
    );
    let expected = [1, 14, 191, 2812, 43238, 674624];
    start_search(&mut pos, expected);
}

#[test]
fn tricky_pos_3() {
    init();
    let mut pos = fen::fen(
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"
            .split_ascii_whitespace()
            .collect(),
        Vec::new(),
    );
    let expected = [1, 6, 264, 9467, 422333, 15833292];
    start_search(&mut pos, expected);
}
