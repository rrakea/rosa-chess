use rosa_engine::eval;
use rosa_engine::fen;
use rosa_engine::runtime;

fn eval_test(fen: Vec<&str>, stockfish: i32) {
    runtime::init();
    let mut pos = fen::fen(fen, Vec::new());
    let mut eval = eval::eval(&pos);
    // Since eval is relative to side
    eval *= pos.clr().as_sign() as i32;
    let range = i32::max(stockfish / 2, 200);
    assert!(
        ((stockfish - range)..(stockfish + range)).contains(&eval),
        "Eval not in range. Stockfish: {}, Self: {}", stockfish, eval 
    );
    pos.flip_color();
    let mut flipped_eval = eval::eval(&pos);
    flipped_eval *= pos.clr().as_sign() as i32;
    assert_eq!(eval, flipped_eval, "Eval not the same for flipped color")
}

#[test]
fn start_eval() {
    let fen = fen::START_FEN;
    let stockfish = 7;
    eval_test(fen.to_vec(), stockfish);
}

#[test]
fn up_bishop() {
    let fen = "rn1qkbnr/pp2pppp/2p5/3p4/3PP1Q1/2N5/PPP2PPP/R1B1KBNR b KQkq - 0 4" ;
    let stockfish = 400;
    eval_test(fen.split_ascii_whitespace().collect(), stockfish);
}

#[test]
fn up_queen() {
    let fen = "rnb1kbnr/pp2p3/8/P1pN1pB1/2PP3p/8/P3PPPP/R2QKBNR w KQkq - 0 8";
    let stockfish = 1138;
    eval_test(fen.split_ascii_whitespace().collect(), stockfish);
}