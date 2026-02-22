//! # Search
//! The search function is one of the most important pieces of any chess engine.
//! As such it uses a variety of differnet optimization techinques aiming at making
//! our search as fast as possible
//! ## Effective Branching Factor
//! One metric for measuring the usefullness of certain optimizations is the effective
//! branching factor. Calculated as the nodes at the current depth / nodes of depth - 1.
//! While in practise this can be difficult to compare between different chess engines it
//! is still a useful visualization for what we are trying to optimize for.  
//! Highly optimized chess enginges have a EBF of around 2. In theory this should mean
//! that for every depth the only check 2 moves - the principle variation move (PV move) and one candidate.
//! In practise they search the PV move at full depth and other likely moves at a reducd depth.
//! ## Move Ordering
//! In order for a lot of the optimizations to function properly we have check good moves first.
//! Statically this is done via different heuristic (i.e. killer heuristic, history heuristic)
//! and MVVLVA (most valuable victim, least valuable attacker). Dynamically it is done via iterative deepening.
//! If we hade perfect move odering we could just return our first move. Since statically analysing a move is
//! a lot cheaper than searching it we try to approximate perfect ordering as much as possible
//! ## Optimizations
//! ### Alpha Beta Pruning
//! Alpha Beta pruning is one of the fundamental algorithms of chess engines.
//! It allows to reduce the search tree (effective branching factor) from the average branching
//! of a normal chess position (~ 35 - 40) to roughly the square root.  
//! It does this without cutting any nodes out of the tree that could potentially be relevant.
//! The intuitition is that if we have already found a good counter move to a proposed move (refutation)
//! we dont have to continue searching for better counter moves.
//! ### Negascout
//! Negascout in a combination of the algorithms negamax and scout
//! Negamax is a variation of the classic Minmax algorithm for opposed games
//! Negascout is also known as PVS (Principle variation search). They are functionally equivalent
//! ### Scout
//! Scout assumes that moves  in the move ordering are likely not as good and
//! and therefore searches them in a so called null window (alpha' = -alpha - 1; beta' = -alpha)  
//! As such any move better than the current posited best move will trigger an alpha cutoff
//! which is detected and researched at a normal alpha beta window
//! While researches are costly scout still significantly reduces the branching factor
//! ### Transposition Table
//! For every position we visit we save the result in the so called transposition table.
//! At the start of every position we check if we have already visited the node.
//! If we have (and the searched depth is bigger than ours ) we can just return that result and
//! dont have to check  ourselves  
//! It massivly reduces the amount of nodes that have to be searched.
//! The intuitive reason are transpositions - Position we have already visited in the same search but through
//! a different move ordering.  
//! However there are a variety of different techniques that allow us to make more use of the transposition table.
//! Firstly we dont delete the collected data between moves. Since we have already explored likely moves extensivly
//! this allows us to speed up subsequent searches. If allowed we also run a search of the likely moves during our
//! oponents thinking time (pondering).  
//! Even if our current depth is bigger than the depth of the entry in the transposition table we still gain information
//! from checking the table. The move we thought was best is saved, which massivly improves move ordering.
//! We also gain information regarding the evaluated score which can narrow our search window
//! Additionally Late Move reductions reduce the depth of calls to our search algorithm for unlikely moves, which allows
//! us to reuse previously used calculations even when the original depth was bigger.
//! ### Iterative Deepening
//! Instead of just searching a position for as long as we have time iterative deepening starts searching at depth = 1
//! and increases this by 1 every time search finishes. While this intuitivly might not make much sense
//! using alpha beta pruning and transposition tables achive in practise a massive gain in efficiency.
//! Part of this comes from better moving ordering which massivly improves the effectiveness of alpha
//! beta pruning.
//! ### Null Move Pruning
//! Null move pruning works under the assumption that doing nothing is always worse than doing something.
//! The assumption holds in practise except for very specific scenarios (zugzwang), which occur so few times,
//! that they are not worth considering checking for.  
//! Null move pruning therefor searches using a null move (= doing nothing) before even calculating possible
//! moves in a position.  
//! This allows us to warm up our transposition table and establish a lower bound for what a move in a position
//! should be able to do. This translates into increasing our beta value, which has an effect of the
//! whole subtree.
//! ### Late move reduction
//! If we have good move it stands to reason that we dont have to check later moves as thoroughly as better scored moves.
//! As described above this also allows to "underbid" the depth of previous searches and massivly gain from
//! transposition table entries.  
//! It is important to remeber than this reductions happens at every depth, as such moves that statically evaluate as bad
//! get searched quite shallowly.   
//! There are a lot of formulas ans heuristic used to decide to what exactly we can reduce our depth.
//! Rosa Chess uses a simple formula of: if depth < 6 {depth - 1} else {depth/3}
//! This formula is definitely open to changes with further testing
//! ## Node Types

use crate::eval;
use crate::make;
use crate::make::Legal;
use crate::mv::mv_gen;
use crate::quiscence::quiscence_search;
use crate::thread_search::*;

use rosa_lib::history;
use rosa_lib::mv::Mv;
use rosa_lib::piece::*;
use rosa_lib::pos;
use rosa_lib::tt;

use std::sync::mpsc;

pub static TT: tt::TT = tt::TT::new();

/// Iterative deepening
pub fn search(mut p: pos::Pos, sender: mpsc::Sender<ThreadReport>, stop: Stop) {
    let mut depth = 0;

    loop {
        depth += 1;

        let score;
        let mut best_mv;
        let mut ponder = None;
        let mut search_stats = SearchStats::new(depth);

        match negascout(
            &mut p,
            depth,
            eval::SAFE_MIN_SCORE,
            eval::SAFE_MAX_SCORE,
            &mut search_stats,
            &stop,
        ) {
            SearchRes::TimeOut => {
                return;
            }
            SearchRes::Node(mv, pon, s) => {
                score = s;
                best_mv = mv;
                ponder = Some(pon);
            }
            SearchRes::Leaf(s) => {
                panic!("Root is a leaf Node. Score: {}", s)
            }
            SearchRes::NoPonderNode(mv, s) => {
                score = s;
                best_mv = mv;
            }
        }

        // Only sende no ponder move is there really is no legal move
        if ponder.is_none() {
            let mut p = p.clone();
            let (_, guard) = make::make(&mut p, &mut best_mv, false);
            unsafe {
                guard.verified_drop();
            }
            // Ponder is in TT
            if let Some(entry) = TT.get(p.key())
                && entry.key == p.key()
            {
                ponder = Some(entry.mv);
            } else {
                // If there are no legal moves -> ponder = none
                // Will return since we are mate / stalemate in 1
                ponder = mv_gen::gen_mvs(&p).pop();
            }
        }

        sender
            .send(ThreadReport::new(
                depth,
                score,
                best_mv,
                ponder,
                search_stats.clone(),
            ))
            .unwrap();

        if score == eval::SAFE_MIN_SCORE || score == eval::SAFE_MAX_SCORE || score == 0 {
            // If the TT entry for the current position is at the current depth
            // -> So we dont spin infinitly on a small tree
            if let Some(entry) = TT.get(p.key())
                && depth > 5
                && entry.depth <= depth
            {
                return;
            }
        }
    }
}

enum SearchRes {
    TimeOut,
    Leaf(i32),
    NoPonderNode(Mv, i32),
    Node(Mv, Mv, i32),
}

impl SearchRes {
    fn from_mvs(mv: (Mv, Option<Mv>), score: i32) -> SearchRes {
        match mv.1 {
            Some(p) => SearchRes::Node(mv.0, p, score),
            None => SearchRes::NoPonderNode(mv.0, score),
        }
    }
}

/// Main search functions; uses the optimizations described above
fn negascout(
    p: &mut pos::Pos, depth: u8, mut alpha: i32, mut beta: i32, stats: &mut SearchStats,
    stop: &Stop,
) -> SearchRes {
    stats.node();
    if p.repetitions() > 2 {
        return SearchRes::Leaf(0);
    }

    if depth == 0 {
        return SearchRes::Leaf(quiscence_search(p, alpha, beta));
    }

    let replace_entry;
    let mut tt_mv = None;
    match parse_tt(p.key(), depth, &mut alpha, &mut beta) {
        TtRes::Miss(replace) => replace_entry = replace,
        TtRes::MvHint(mv, replace) => {
            stats.tt_hit();
            replace_entry = replace;
            tt_mv = Some(mv);
        }
        TtRes::Cutoff(score, mv) => {
            stats.tt_hit();
            // If we are in a pv node we dont want to cut on tt
            if beta - alpha == 1 {
                return SearchRes::NoPonderNode(mv, score);
            }
            // We are in PV
            tt_mv = Some(mv);
            replace_entry = false;
        }
    }

    if depth < 5 && stop.is_done() {
        return SearchRes::TimeOut;
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
            SearchRes::TimeOut => {
                make::unmake(p, pv, pv_guard);
                return SearchRes::TimeOut;
            }
            SearchRes::Node(ponder, _, s) | SearchRes::NoPonderNode(ponder, s) => {
                best_mvs = (pv, Some(ponder));
                score = -s;
            }
            SearchRes::Leaf(s) => score = -s,
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
            return SearchRes::from_mvs(best_mvs, alpha);
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
                SearchRes::TimeOut => {
                    make::unmake(p, m, make_guard);
                    return SearchRes::TimeOut;
                }
                SearchRes::Node(res, _, s) | SearchRes::NoPonderNode(res, s) => {
                    response = Some(res);
                    score = -s;
                }
                SearchRes::Leaf(s) => score = -s,
            }
        } else {
            // Not reduced depth null window
            match negascout(p, depth - 1, -alpha - 1, -alpha, stats, stop) {
                SearchRes::TimeOut => {
                    make::unmake(p, m, make_guard);
                    return SearchRes::TimeOut;
                }
                SearchRes::Node(res, _, s) | SearchRes::NoPonderNode(res, s) => {
                    response = Some(res);
                    score = -s;
                }
                SearchRes::Leaf(s) => score = -s,
            }
        }

        if alpha < score && score < beta {
            // Unstable Node -> Dont do LMR
            lmr_stable = false;
            // Failed high -> Full re-search
            match negascout(p, depth - 1, -beta, -score, stats, stop) {
                SearchRes::TimeOut => {
                    make::unmake(p, m, make_guard);
                    return SearchRes::TimeOut;
                }
                SearchRes::Node(res, _, s) | SearchRes::NoPonderNode(res, s) => {
                    response = Some(res);
                    score = -s;
                }
                SearchRes::Leaf(s) => score = -s,
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
    return SearchRes::from_mvs(best_mvs, alpha);
}

#[inline(always)]
fn no_legal_moves(p: &pos::Pos) -> SearchRes {
    let king_pos = p.piece(Piece::King.clr(p.clr())).get_ones_single();
    if !make::square_attacked(p, p.clr(), king_pos) {
        // Stalemate
        return SearchRes::Leaf(0);
    } else {
        // Checkmate
        return SearchRes::Leaf(eval::SAFE_MIN_SCORE);
    }
}

#[inline(always)]
fn do_null_move(
    p: &mut pos::Pos, depth: u8, beta: i32, tt_mv: Option<Mv>, stats: &mut SearchStats, stop: &Stop,
) -> Option<SearchRes> {
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
        SearchRes::TimeOut => {
            make::unmake_null(p, was_ep, null_guard);
            return Some(SearchRes::TimeOut);
        }
        SearchRes::Node(_, _, score)
        | SearchRes::Leaf(score)
        | SearchRes::NoPonderNode(_, score) => null_score = -score,
    }

    // The null move sets the baseline for what we think we can achive
    // Even if we dont make a move we are still outside of the window
    make::unmake_null(p, was_ep, null_guard);
    if null_score >= beta {
        match tt_mv {
            Some(m) => return Some(SearchRes::NoPonderNode(m, beta)),
            None => return Some(SearchRes::Leaf(beta)),
        }
    }
    return None;
}

enum TtRes {
    Miss(bool),
    Cutoff(i32, Mv),
    MvHint(Mv, bool),
}

/// Reading from the transposition table.
/// Split into its own function to decrease complexity of the negascout function
#[inline(always)]
fn parse_tt(key: tt::Key, depth: u8, alpha: &mut i32, beta: &mut i32) -> TtRes {
    let entry = match TT.get(key) {
        None => return TtRes::Miss(true),
        Some(e) => e,
    };

    if entry.key != key {
        let replace = entry.depth < depth;
        return TtRes::Miss(replace);
    }

    if entry.depth < depth {
        // The Entry knows less than we want to known
        // -> Still use PV move for move ordering
        return TtRes::MvHint(entry.mv, true);
    }

    match entry.node_type {
        // The Node is at a greater depth && exact -> Just use that value
        tt::EntryType::Exact => return TtRes::Cutoff(entry.score, entry.mv),
        tt::EntryType::Upper => {
            if entry.score <= *alpha {
                return TtRes::Cutoff(entry.score, entry.mv);
            }
            // We have a better upper bound
            if entry.score < *beta {
                *beta = entry.score;
            }
            return TtRes::MvHint(entry.mv, false);
        }

        tt::EntryType::Lower => {
            if entry.score >= *beta {
                return TtRes::Cutoff(entry.score, entry.mv);
            }
            // We have a better lower bound
            if entry.score > *alpha {
                *alpha = entry.score;
            }
            return TtRes::MvHint(entry.mv, false);
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

pub fn debug_division_search(p: &mut pos::Pos, depth: u8) {
    let mut total = 0;
    let mut moves = Vec::new();

    for mut mv in mv_gen::gen_mvs(p) {
        let (legal, guard) = make::make(p, &mut mv, true);
        make::unmake(p, mv, guard);
        if legal == make::Legal::ILLEGAL {
            continue;
        }

        let count = div_search_helper(p, depth - 1);
        total += count;
        moves.push(format!("{}: {}", mv, count));
    }

    moves.sort();
    for m in moves {
        print!("{m}\n");
    }
    println!("Total: {total}")
}

fn div_search_helper(p: &mut pos::Pos, depth: u8) -> u64 {
    if depth <= 0 {
        return 1;
    }

    let mut total = 0;
    for mut mv in mv_gen::gen_mvs(p) {
        let (legal, guard) = make::make(p, &mut mv, true);
        make::unmake(p, mv, guard);
        if legal == make::Legal::ILLEGAL {
            continue;
        }

        total += div_search_helper(p, depth - 1);
    }

    total
}
