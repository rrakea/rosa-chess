use std::sync::Once;

use rosa_lib::mv::*;
use rosa_lib::mvvlva::init_mvvlva;
use rosa_lib::piece::*;

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| init_mvvlva());
}

#[test]
fn cap() {
    init();
    let m = Mv::new_cap(3, 42, Piece::Bishop, Piece::Rook);
    assert_eq!(m.sq(), (3, 42));
    assert_eq!(m.cap_capturer(), Piece::Bishop);
    assert_eq!(m.cap_victim(), Piece::Rook);
    assert_eq!(m.flag(), Flag::Cap);
}

#[test]
fn prom() {
    init();
    let m = Mv::new_prom(54, 32, Piece::Knight);
    assert_eq!(m.sq(), (54, 32));
    assert_eq!(m.prom_piece(), Piece::Knight);
    assert_eq!(m.flag(), Flag::Prom);
}

#[test]
fn promcap() {
    init();
    let m = Mv::new_prom_cap(41, 0, Piece::Queen, Piece::Rook);
    assert_eq!(m.sq(), (41, 0));
    assert_eq!(m.prom_piece(), Piece::Queen);
    assert_eq!(m.cap_capturer(), Piece::Pawn);
    assert_eq!(m.cap_victim(), Piece::Rook);
    assert_eq!(m.flag(), Flag::PromCap);
}

#[test]
fn double() {
    init();
    let m = Mv::new_double(5, 32);
    assert_eq!(m.sq(), (5, 32));
    assert_eq!(m.flag(), Flag::Double);
}

#[test]
fn ep() {
    init();
    let m = Mv::new_ep(2, 42);
    assert_eq!(m.sq(), (2, 42));
    assert_eq!(m.flag(), Flag::Ep);
    assert_eq!(m.cap_capturer(), Piece::Pawn);
    assert_eq!(m.cap_victim(), Piece::Pawn);
}
