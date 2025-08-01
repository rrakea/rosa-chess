use std::collections::HashMap;
use std::sync::Once;

use rosa::mv::{self, magic_init};
use rosa::*;

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
        table::init_zobrist_keys();
        magic_init::init_magics();
    });
}

fn counting_search(p: &pos::Pos, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut count: u64 = 0;
    let mv_iter = mv::mv_gen::gen_mvs(p).filter(|mv| !mv.is_null());
    for mv in mv_iter {
        let npos = mv::mv_apply::apply(p, &mv);
        let npos = match npos {
            Some(n) => n,
            None => continue,
        };
        count += counting_search(&npos, depth - 1);
    }
    count
}

fn division_test(map: HashMap<&str, u32>, depth: usize) {
    let mut total = 0;
    let mut expected_total = 0;
    let pos = fen::starting_pos();
    let mut depth_two_mv = Vec::new();
    for mv in mv::mv_gen::gen_mvs(&pos).filter(|mv| !mv.is_null()) {
        let npos = mv::mv_apply::apply(&pos, &mv);
        let npos = match npos {
            Some(p) => p,
            None => continue,
        };
        depth_two_mv.push((npos, mv));
    }
    println!();
    for (pos, mv) in depth_two_mv.iter() {
        let count = counting_search(pos, depth);
        total += count;
        let expected = map.get(mv.notation().as_str());
        let expected = match expected {
            Some(e) => e,
            None => {
                println!("Not found in map: {}", mv.notation());
                continue;
            }
        };
        expected_total += expected;

        println!(
            "Division: Mv1: {}, Count: {}, Expected: {}",
            mv.notation(),
            count,
            expected,
        );
    }
    println!("Division Total: {total}, Expected: {expected_total}")
}

fn all_possible_moves(fen: String) {
    let pos = fen::fen(fen);
    println!("{}", pos.prittify());
    let mut count = 0;
    for mv in mv::mv_gen::gen_mvs(&pos).filter(|mv| !mv.is_null()) {
        println!("{}", mv.prittify());
        count += 1;
    }
    println!("Total {count}");
}

//#[test]
fn starting_pos() {
    init();
    let pos = fen::starting_pos();
    let start_values: [u64; 9] = [
        1,
        20,
        400,
        8902,
        197281,
        4865609,
        119060324,
        3195901860,
        84998978956,
    ];

    for (i, r) in start_values.iter().enumerate() {
        println!(
            "Perft: At depth: {i}, Got: {}, Expected: {}",
            counting_search(&pos, i),
            *r
        );
    }
}

#[test]
fn division_2() {
    init();
    let expected_values = HashMap::from([
        ("a2a3", 380),
        ("b2b3", 420),
        ("c2c3", 420),
        ("d2d3", 539),
        ("e2e3", 599),
        ("f2f3", 380),
        ("g2g3", 420),
        ("h2h3", 380),
        ("a2a4", 420),
        ("b2b4", 421),
        ("c2c4", 441),
        ("d2d4", 560),
        ("e2e4", 600),
        ("f2f4", 401),
        ("g2g4", 421),
        ("h2h4", 420),
        ("b1a3", 400),
        ("b1c3", 440),
        ("g1f3", 440),
        ("g1h3", 400),
    ]);
    division_test(expected_values, 2);
}

#[test]
fn division_1() {
    init();
    let map = HashMap::from([
        ("a2a3", 20),
        ("b2b3", 20),
        ("c2c3", 20),
        ("d2d3", 20),
        ("e2e3", 20),
        ("f2f3", 20),
        ("g2g3", 20),
        ("h2h3", 20),
        ("a2a4", 20),
        ("b2b4", 20),
        ("c2c4", 20),
        ("d2d4", 20),
        ("e2e4", 20),
        ("f2f4", 20),
        ("g2g4", 20),
        ("h2h4", 20),
        ("b1a3", 20),
        ("b1c3", 20),
        ("g1f3", 20),
        ("g1h3", 20),
    ]);
    division_test(map, 1);
}

//#[test]
fn tricky_pos_1() {
    init();
    all_possible_moves(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 b - - 0 10".to_string(),
    );
}

#[test]
fn tricky_pos_2() {
    init();
    all_possible_moves(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".to_string(),
    );
}
