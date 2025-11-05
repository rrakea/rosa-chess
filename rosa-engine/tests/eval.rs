use rosa_engine::eval;
use rosa_engine::fen;
use rosa_engine::runtime;

fn eval_test(fen: Vec<&str>, stockfish: i32) {
    runtime::init();
    let mut pos = fen::fen(fen, Vec::new());
    let eval = eval::eval(&pos);
    let range = 200;
    assert!(
        ((stockfish - range)..(stockfish + range)).contains(&eval),
        "Eval not in range"
    );
    pos.flip_color();
    let flipped_eval = eval::eval(&pos);
    assert_eq!(eval, -flipped_eval, "Eval not the same for flipped color")
}

#[test]
fn start_eval() {
    let fen = fen::START_FEN;
    let stockfish = 7;
    eval_test(fen.to_vec(), stockfish);
}
