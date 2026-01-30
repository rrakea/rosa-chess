use crate::thread_search::Stop;
use crate::{make, make::Legal, quiscence::quiscence_search};
use rosa_lib::{history, mv::Mv, pos, pos::Pos, tt};

enum Res {
    TimeOut,
    Leaf(i32),
    NoPonderNode(Mv, i32),
    Node(Mv, Mv, i32),
}

pub struct Data {
    max_depth: u8,
    nodes: u64,
    tt_hits: u64,
    timeout_nodes: u64,
    stop: Stop,
}

pub static TT: tt::TT = tt::TT::new();

pub fn search(p: &mut Pos, depth: u8, mut alpha: i32, mut beta: i32, data: &mut Data) -> Res {
    data.node();
    if data.timeout() {
        return Res::TimeOut;
    }

    if p.repetitions() > 2 {
        return Res::Leaf(0);
    }

    if depth == 0 {
        return Res::Leaf(quiscence_search(p, alpha, beta));
    }

    let (replace_entry, tt_mv, return_val) = parse_tt(p.key(), depth, &mut alpha, &mut beta);

    if let Some(_) = tt_mv {
        stats.tt_hit();
    }
    if let Some(rv) = return_val {
        return Res::from_tt(tt_mv, rv);
    }

    let null_mv_return = do_null_move(p, depth, beta, tt_mv, stats, stop);
    if let Some(res) = null_mv_return {
        return res;
    }

    // mv_gen is only called if tt_mv == None
    // -> If tt mv produces a cutoff, we never do mv_gen
    let mut iter = get_mv_iter(p, tt_mv).into_iter();

    let mut score;
    let mut best_mvs: (Mv, Option<Mv>);
    let mut node_type = tt::EntryType::Upper;

    // Only the first move!
    loop {
        let mut pv = match iter.next() {
            Some(mv) => mv,
            None => return no_legal_moves(p),
        };
        // Process PV move
        let (legal, pv_guard) = make::make(p, &mut pv, true);
        if legal == Legal::ILLEGAL {
            make::unmake(p, pv, pv_guard);
            continue;
        }

        best_mvs = (pv, None);

        match negascout(p, depth - 1, -beta, -alpha, stats, stop) {
            Res::TimeOut => {
                make::unmake(p, pv, pv_guard);
                return Res::TimeOut;
            }
            Res::Node(ponder, _, s) | Res::NoPonderNode(ponder, s) => {
                best_mvs = (pv, Some(ponder));
                score = -s;
            }
            Res::Leaf(s) => score = -s,
        }
        make::unmake(p, pv, pv_guard);

        if score > alpha {
            alpha = score;
            node_type = tt::EntryType::Exact;
        }

        if score >= beta {
            history::set(&pv, p.clr(), depth);

            if replace_entry {
                TT.set(tt::Entry::new(
                    p.key(),
                    alpha,
                    pv,
                    depth,
                    tt::EntryType::Lower,
                ));
            }
            return Res::from_mvs(best_mvs, alpha);
        }

        break;
    }

    // Check the rest of the moves using scout
    let mut lmr_stable = true;
    for (i, mut m) in iter.enumerate() {
        let (legal, make_guard) = make::make(p, &mut m, true);
        if legal == make::Legal::ILLEGAL {
            make::unmake(p, m, make_guard);
            continue;
        }

        let mut response = None;
        // Null window search & Late move reduction
        if do_lmr(lmr_stable, depth, i) {
            let reduced_depth = late_move_reduction(depth);
            match negascout(p, reduced_depth, -alpha - 1, -alpha, stats, stop) {
                Res::TimeOut => {
                    make::unmake(p, m, make_guard);
                    return Res::TimeOut;
                }
                Res::Node(res, _, s) | Res::NoPonderNode(res, s) => {
                    response = Some(res);
                    score = -s;
                }
                Res::Leaf(s) => score = -s,
            }
        } else {
            // Not reduced depth null window
            match negascout(p, depth - 1, -alpha - 1, -alpha, stats, stop) {
                Res::TimeOut => {
                    make::unmake(p, m, make_guard);
                    return Res::TimeOut;
                }
                Res::Node(res, _, s) | Res::NoPonderNode(res, s) => {
                    response = Some(res);
                    score = -s;
                }
                Res::Leaf(s) => score = -s,
            }
        }

        if alpha < score && score < beta {
            // Unstable Node -> Dont do LMR
            lmr_stable = false;
            // Failed high -> Full re-search
            match negascout(p, depth - 1, -beta, -score, stats, stop) {
                Res::TimeOut => {
                    make::unmake(p, m, make_guard);
                    return Res::TimeOut;
                }
                Res::Node(res, _, s) | Res::NoPonderNode(res, s) => {
                    response = Some(res);
                    score = -s;
                }
                Res::Leaf(s) => score = -s,
            }
        }

        if score > alpha {
            alpha = score;
            best_mvs = (m, response);
            node_type = tt::EntryType::Exact;
        }

        if score >= beta {
            // Cut Node
            node_type = tt::EntryType::Lower;
            make::unmake(p, m, make_guard);
            history::set(&m, p.clr(), depth);
            break; // Prune :)
        }

        make::unmake(p, m, make_guard);
    }

    if replace_entry {
        TT.set(tt::Entry::new(p.key(), alpha, best_mvs.0, depth, node_type));
    }
    return Res::from_mvs(best_mvs, alpha);
}

#[inline(always)]
fn no_legal_moves(p: &pos::Pos) -> Res {
    let king_pos = p.piece(Piece::King.clr(p.clr())).get_ones_single();
    if !make::square_attacked(p, p.clr(), king_pos) {
        // Stalemate
        return Res::Leaf(0);
    } else {
        // Checkmate
        return Res::Leaf(eval::SAFE_MIN_SCORE);
    }
}

#[inline(always)]
fn do_null_move(
    p: &mut pos::Pos,
    depth: u8,
    beta: i32,
    tt_mv: Option<Mv>,
    stats: &mut SearchStats,
    stop: &Stop,
) -> Option<Res> {
    if depth < 4 {
        return None;
    }

    let (legal, was_ep, null_guard) = make::make_null(p);
    if legal == make::Legal::ILLEGAL {
        make::unmake_null(p, was_ep, null_guard);
        return None;
    }

    let null_score;
    match negascout(p, depth - 3, -beta, -(beta - 1), stats, stop) {
        Res::TimeOut => {
            make::unmake_null(p, was_ep, null_guard);
            return Some(Res::TimeOut);
        }
        Res::Node(_, _, score) | Res::Leaf(score) | Res::NoPonderNode(_, score) => {
            null_score = -score
        }
    }

    // The null move sets the baseline for what we think we can achive
    // Even if we dont make a move we are still outside of the window
    make::unmake_null(p, was_ep, null_guard);
    if null_score >= beta {
        return Some(Res::from_tt(tt_mv, beta));
    }
    return None;
}

/// Reading from the transposition table.
/// Split into its own function to decrease complexity of the negascout function
#[inline(always)]
fn parse_tt(
    key: tt::Key,
    depth: u8,
    alpha: &mut i32,
    beta: &mut i32,
) -> (bool, Option<Mv>, Option<i32>) {
    let entry = TT.get(key);
    match entry {
        None => {
            return (true, None, None);
        }
        Some(entry) => {
            if entry.key != key {
                let replace = entry.depth < depth;
                return (replace, None, None);
            }

            let pv_move = Some(entry.mv);
            let mut return_val = None;
            if entry.depth < depth {
                // The Entry knows less than we want to known
                // -> Still use PV move for move ordering
                return (true, pv_move, return_val);
            }

            match entry.node_type {
                // The Node is at a greater depth && exact -> Just use that value
                tt::EntryType::Exact => return (false, pv_move, Some(entry.score)),
                tt::EntryType::Upper => {
                    if entry.score <= *alpha {
                        return_val = Some(entry.score);
                    } else {
                        *beta = i32::min(entry.score, *beta);
                    }
                }
                tt::EntryType::Lower => {
                    if entry.score >= *beta {
                        return_val = Some(entry.score);
                    } else {
                        *alpha = i32::max(entry.score, *alpha);
                    }
                }
            }

            if alpha >= beta {
                return_val = Some(entry.score);
            }
            (false, pv_move, return_val)
        }
    }
}

/// Get the move iter depending on the move we got from the transposition table
/// -> We have to exclude it if tt lookup was succesful
#[inline(always)]
fn get_mv_iter(p: &pos::Pos, best_mv: Option<Mv>) -> Box<dyn Iterator<Item = Mv>> {
    // Generating the move iter
    match best_mv {
        None => Box::new(
            mv_gen::gen_mvs_stages(p, true)
                .into_iter()
                .chain(mv_gen::gen_mvs_stages(p, false)),
        ),
        // Since we dont need to check the non cap mvs if pv is a cap
        Some(pv) => match pv.is_cap() {
            true => Box::new(
                std::iter::once(pv).chain(
                    mv_gen::gen_mvs_stages(p, true)
                        .into_iter()
                        .filter(move |m| m != &pv)
                        .chain(mv_gen::gen_mvs_stages(p, false)),
                ),
            ),
            false => Box::new(
                std::iter::once(pv).chain(
                    mv_gen::gen_mvs_stages(p, true).into_iter().chain(
                        mv_gen::gen_mvs_stages(p, false)
                            .into_iter()
                            .filter(move |m| m != &pv),
                    ),
                ),
            ),
        },
    }
}

/// How many moves to do before starting late move reductions
const LMR_MOVES: usize = 2;

/// Calc LMR reduction depending on depth
/// Very basic right now; subject to change
#[inline(always)]
fn late_move_reduction(depth: u8) -> u8 {
    if depth < 6 {
        depth - 1
    } else {
        depth - (depth / 3)
    }
}

#[inline(always)]
fn do_lmr(do_lmr: bool, depth: u8, i: usize) -> bool {
    do_lmr && depth > 2 && i >= LMR_MOVES
}
impl Res {
    fn from_mvs(mv: (Mv, Option<Mv>), score: i32) -> Res {
        match mv.1 {
            Some(p) => Res::Node(mv.0, p, score),
            None => Res::NoPonderNode(mv.0, score),
        }
    }
    fn from_tt(mv: Option<Mv>, score: i32) -> Res {
        match mv {
            Some(m) => Res::NoPonderNode(m, score),
            None => Res::Leaf(score),
        }
    }
}

const TIMEOUT_NODES: u64 = 4098;

impl Data {
    pub fn new(depth: u8, stop: Stop) -> Data {
        Data {
            max_depth: depth,
            nodes: 0,
            tt_hits: 0,
            timeout_nodes: 0,
            stop: stop,
        }
    }

    fn node(&mut self) {
        self.nodes += 1;
        self.timeout_nodes += 1;
    }

    fn timeout(&mut self) -> bool {
        if self.timeout_nodes >= TIMEOUT_NODES {
            if self.stop.is_done() {
                return true;
            }
            self.timeout_nodes = 0;
        }
        return false;
    }

    fn tt_hit(&mut self) {
        self.tt_hits += 1;
    }
}
