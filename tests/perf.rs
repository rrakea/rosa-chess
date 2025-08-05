use rosa::*;
use std::sync::Once;

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
        table::init_zobrist_keys();
        mv::magic_init::init_magics();
    });
}

fn count(p: &pos::Pos, expected: [u64; 6]) {
    let tt = table::TT::new(10000);
    for (i, res) in expected.iter().enumerate() {
        let count = search::counting_search(p, i as u8, tt, true);
        assert_eq!(count, *res);
    }
}

#[test]
fn starting_pos() {
    init();
    let pos = fen::starting_pos();
    let expected = [1, 20, 400, 8902, 197281, 4865609];
    count(&pos, expected);
}

#[test]
fn tricky_pos_1() {
    init();
    let pos =
        fen::fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ".to_string());
    let expected = [1, 48, 2039, 97862, 4085603, 193690690];
    count(&pos, expected);
}

#[test]
fn tricky_pos_2() {
    init();
    let pos = fen::fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ".to_string());
    let expected = [1, 14, 191, 2812, 43238, 674624];
    count(&pos, expected);
}

#[test]
fn tricky_pos_3() {
    init();
    let pos =
        fen::fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_string());
    let expected = [1, 6, 264, 9467, 422333, 15833292];
    count(&pos, expected);
}
