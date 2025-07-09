use super::pos;

const FULL: u64 = u64::MAX;

fn init_bboards(p: &pos::Pos) {}

pub fn bb_all(p: &pos::Pos) -> u64 {
    p.wp | p.wn | p.wb | p.wr | p.wq | p.wk | p.bp | p.bn | p.bb | p.br | p.bq | p.bk
}

pub fn bb_w(p: &pos::Pos) -> u64 {
    p.wp | p.wn | p.wb | p.wr | p.wq | p.wk
}

pub fn bb_b(p: &pos::Pos) -> u64 {
    p.bp | p.bn | p.bb | p.br | p.bq | p.bk
}

pub fn none(p: &pos::Pos) -> u64 {
    bb_all(p) ^ FULL
}
