use rosa_lib::long_mv::LongMv;
use rosa_lib::pos::Pos;

/*
    Problems:
        - Is Double
        - Is castle
*/

pub fn make(p: &mut Pos, mv: &LongMv) {
    let color = p.active;
    p.flip_color();

    let start = mv.start();
    let end = mv.end();
    let mut op_end = end;

    if mv.is_ep() {
        op_end = if color == 1 { end - 8 } else { end + 8 };
    }

    let piece = p.piece_at_sq(start);
    let op_piece = mv.captured_piece();

    p.piece_toggle(piece, sq);

    if !mv.is_prom() {
        p.piece_toggle(piece, end);
    }

    if mv.is_prom() {
        let prom_piece = mv.prom_piece();
        p.piece_toggle(prom_piece, sq);
    }
}
