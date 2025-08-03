use std::sync::Once;
use rosa::*;

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
        table::init_zobrist_keys();
        mv::magic_init::init_magics();
    });
}

#[test]
fn starting_pos() {
    init();
    let pos = fen::starting_pos();
    let start_values: [u64; 6] = [
        1,
        20,
        400,
        8902,
        197281,
        4865609,
    ];

    for (i, r) in start_values.iter().enumerate() {
        println!(
            "Perft: At depth: {i}, Got: {}, Expected: {}",
            search::counting_search(&pos, i as u8),
            *r
        );
    }
}

#[test]
fn apply_test() {
    init();
    let pos = fen::starting_pos();
    let pos = mv::mv_apply::apply(&pos, &mv::mv::Mv::new(12, 28, mv::mv::MvFlag::Quiet)).unwrap();
    let pos = mv::mv_apply::apply(&pos, &mv::mv::Mv::new(55, 39, mv::mv::MvFlag::DoubleP)).unwrap();
    let pos = mv::mv_apply::apply(&pos, &mv::mv::Mv::new(3, 39, mv::mv::MvFlag::Cap)).unwrap();
    let pos = mv::mv_apply::apply(&pos, &mv::mv::Mv::new(51, 43, mv::mv::MvFlag::Quiet)).unwrap();
    let pos = mv::mv_apply::apply(&pos, &mv::mv::Mv::new(1, 18, mv::mv::MvFlag::Quiet)).unwrap();
    let pos = mv::mv_apply::apply(&pos, &mv::mv::Mv::new(58, 37, mv::mv::MvFlag::Quiet)).unwrap();
    let pos = mv::mv_apply::apply(&pos, &mv::mv::Mv::new(28, 37, mv::mv::MvFlag::Cap)).unwrap();
    let pos = mv::mv_apply::apply(&pos, &mv::mv::Mv::new(52, 44, mv::mv::MvFlag::Quiet)).unwrap();
    println!("{}", pos.prittify());
}
