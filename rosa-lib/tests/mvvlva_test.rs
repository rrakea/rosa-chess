use std::sync::Once;

use rosa_lib::{mvvlva::*, piece::Piece};

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| init_mvvlva());
}

#[test]
fn pawn() {
    init();
    let d = compress(Piece::Pawn, Piece::Knight);
    assert_eq!(decompress(d), (Piece::Pawn, Piece::Knight));
}

#[test]
fn king() {
    init();
    let d = compress(Piece::King, Piece::Pawn);
    assert_eq!(decompress(d), (Piece::King, Piece::Pawn));
}

#[test]
fn rook() {
    init();
    let d = compress(Piece::Bishop, Piece::Rook);
    assert_eq!(decompress(d), (Piece::Bishop, Piece::Rook));
}
