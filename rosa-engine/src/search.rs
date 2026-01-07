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
use crate::mv::mv_gen;
use crate::thread_search;

use rosa_lib::history;
use rosa_lib::mv::Mv;
use rosa_lib::piece::*;
use rosa_lib::pos;
use rosa_lib::tt;

use std::sync::mpsc;

pub static TT: tt::TT = tt::TT::new();

/// Iterative deepening
pub fn search(mut p: pos::Pos, sender: mpsc::Sender<thread_search::ThreadReport>) {
    // Iterative deepening
    let mut depth = 0;
    let mut score;
    let mut best_mv;

    loop {
        depth += 1;
        let mut search_stats = thread_search::SearchStats::new(depth);

        match negascout(&mut p, depth, eval::SAFE_MIN_SCORE, eval::SAFE_MAX_SCORE, &mut search_stats) {
            SearchRes::TimeOut => {
                return;
            }
            SearchRes::Node(s) => {
                score = s;
                best_mv = TT.get(p.key()).unwrap().mv;
            }
        }

        sender
            .send(thread_search::ThreadReport::new(
                depth,
                score,
                best_mv,
                search_stats,
            ))
            .unwrap();
    }
}

/// How many moves to do before starting late move reductions
const LMR_MOVES: usize = 2;

enum SearchRes {
    TimeOut,
    Node(i32),
}

/// Main search functions; uses the optimizations described above
fn negascout(
    p: &mut pos::Pos,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    stats: &mut thread_search::SearchStats,
) -> SearchRes {
    stats.node();
    if depth == 0 {
        return SearchRes::Node(eval::eval(p));
    }

    let (replace_entry, mut best_mv, return_val) = parse_tt(p.key(), depth, &mut alpha, &mut beta);

    if let Some(_) = best_mv {
        stats.tt_hit();
        if let Some(rv) = return_val {
            return SearchRes::Node(rv);
        }
    }

    if depth < 5 && thread_search::search_done() {
        return SearchRes::TimeOut;
    }

    // Null Move
    if depth > 3 {
        let (legal, was_ep, null_guard) = make::make_null(p);
        if legal == make::Legal::LEGAL {
            let null_score = {
                match negascout(p, depth - 3, -beta, -(beta - 1), stats) {
                    SearchRes::TimeOut => {
                        make::unmake_null(p, was_ep, null_guard);
                        return SearchRes::TimeOut;
                    }
                    SearchRes::Node(score) => -score,
                }
            };

            // The null move sets the baseline for what we think we can achive
            // Even if we dont make a move we are still outside of the window
            if null_score >= beta {
                make::unmake_null(p, was_ep, null_guard);
                return SearchRes::Node(beta);
            }
        }
        make::unmake_null(p, was_ep, null_guard);
    }

    let mut node_type = tt::EntryType::Upper;
    let mut first_iteration = true;

    // Process PV move
    if let Some(mut m) = best_mv {
        first_iteration = false;

        let score;
        let (_legal, pv_guard) = make::make(p, &mut m, false);
        match negascout(p, depth - 1, -beta, -alpha, stats) {
            SearchRes::TimeOut => {
                make::unmake(p, &mut m, pv_guard);
                return SearchRes::TimeOut;
            }
            SearchRes::Node(s) => score = -s,
        }
        make::unmake(p, &mut m, pv_guard);

        if score > alpha {
            alpha = score;
            node_type = tt::EntryType::Exact;
        }

        if score >= beta {
            history::set(&m, p.clr(), depth);

            if replace_entry {
                TT.set(tt::Entry::new(
                    p.key(),
                    alpha,
                    m,
                    depth,
                    tt::EntryType::Lower,
                ));
            }
            return SearchRes::Node(alpha);
        }
    }

    // Generating the move iter
    let iter: Box<dyn Iterator<Item = Mv>> = match best_mv {
        None => Box::new(
            mv_gen::gen_mvs_stages(p, true)
                .into_iter()
                .chain(mv_gen::gen_mvs_stages(p, false)),
        ),
        // Since we dont need to check the non cap mvs if pv is a cap
        Some(pv) => match pv.is_cap() {
            true => Box::new(
                mv_gen::gen_mvs_stages(p, true)
                    .into_iter()
                    .filter(move |m| m != &pv)
                    .chain(mv_gen::gen_mvs_stages(p, false)),
            ),
            false => Box::new(
                mv_gen::gen_mvs_stages(p, true).into_iter().chain(
                    mv_gen::gen_mvs_stages(p, false)
                        .into_iter()
                        .filter(move |m| m != &pv),
                ),
            ),
        },
    };

    let mut do_lmr = true;

    for (i, mut m) in iter.enumerate() {
        let (legal, make_guard) = make::make(p, &mut m, true);
        if legal == make::Legal::ILLEGAL {
            make::unmake(p, &mut m, make_guard);
            continue;
        }

        let mut score;
        if first_iteration {
            first_iteration = false;
            // Principle variation search
            // PV Node
            match negascout(p, depth - 1, -beta, -alpha, stats) {
                SearchRes::TimeOut => {
                    make::unmake(p, &mut m, make_guard);
                    return SearchRes::TimeOut;
                }
                SearchRes::Node(s) => score = -s,
            }
        } else {
            // Null window search
            if depth > 2 && i > LMR_MOVES && do_lmr {
                // Late move reduction
                let reduced_depth = if depth < 6 { depth - 1 } else { depth / 3 };
                match negascout(p, reduced_depth, -alpha - 1, -alpha, stats) {
                    SearchRes::TimeOut => {
                        make::unmake(p, &mut m, make_guard);
                        return SearchRes::TimeOut;
                    }
                    SearchRes::Node(s) => score = -s,
                }
            } else {
                // Not reduced depth null window
                match negascout(p, depth - 1, -alpha - 1, -alpha, stats) {
                    SearchRes::TimeOut => {
                        make::unmake(p, &mut m, make_guard);
                        return SearchRes::TimeOut;
                    }
                    SearchRes::Node(s) => score = -s,
                }
            }
            if alpha < score && score < beta {
                // Unstable Node -> Dont do LMR
                if i <= LMR_MOVES {
                    do_lmr = false;
                }
                // Failed high -> Full re-search
                match negascout(p, depth - 1, -beta, -score, stats) {
                    SearchRes::TimeOut => {
                        make::unmake(p, &mut m, make_guard);
                        return SearchRes::TimeOut;
                    }
                    SearchRes::Node(s) => score = -s,
                }
            }
        }

        if score > alpha {
            alpha = score;
            best_mv = Some(m);
            node_type = tt::EntryType::Exact;
        }

        if score >= beta {
            // Cut Node
            node_type = tt::EntryType::Lower;
            make::unmake(p, &mut m, make_guard);
            history::set(&m, p.clr(), depth);
            break; // Prune :)
        }

        make::unmake(p, &mut m, make_guard);
    }

    match best_mv {
        None => {
            // We never encountered a valid move
            let king_pos = p.piece(Piece::King.clr(p.clr())).get_ones_single();
            if !make::square_attacked(p, p.clr(), king_pos) {
                // Stalemate
                return SearchRes::Node(0);
            } else {
                // Checkmate
                return SearchRes::Node(eval::SAFE_MIN_SCORE);
            }
        }
        Some(best_mv) => {
            if replace_entry {
                TT.set(tt::Entry::new(p.key(), alpha, best_mv, depth, node_type));
            }
        }
    }

    return SearchRes::Node(alpha);
}

/// Reading from the transposition table.
/// Split into its own function to decrease complexity of the negascout function
fn parse_tt(
    key: tt::Key,
    depth: u8,
    alpha: &mut i32,
    beta: &mut i32,
) -> (bool, Option<Mv>, Option<i32>) {
    let mut replace = false;
    let mut pv_move = None;
    let mut return_val = None;

    let entry = TT.get(key);
    match entry {
        None => {
            return (true, None, None);
        }
        Some(entry) => {
            if entry.depth < depth {
                replace = true;
            }

            if entry.key == key {
                pv_move = Some(entry.mv);
                // If the depth is worse we cant use the score
                if depth <= entry.depth {
                    match entry.node_type {
                        tt::EntryType::Exact => {
                            return_val = Some(entry.score);
                        }
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
                }
            }
        }
    }

    (replace, pv_move, return_val)
}
