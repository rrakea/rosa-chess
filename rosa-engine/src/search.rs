use crate::eval;
use crate::make;
use crate::mv;
use crate::mv::mv_gen;
use crate::stats;

use rosa_lib::history;
use rosa_lib::mv::Mv;
use rosa_lib::piece::*;
use rosa_lib::pos;
use rosa_lib::tt;

use std::sync::RwLock;
use std::thread;
use std::time;

pub static TT: tt::TT = tt::TT::new();
static STOP: RwLock<bool> = RwLock::new(false);

pub fn thread_search(p: &pos::Pos) {
    stats::reset_node_count();
    *STOP.write().unwrap() = false;
    let pclone = p.clone();
    // The thread kills itself when it gets the stop signal
    thread::spawn(move || search(pclone));
}

pub fn search(mut p: pos::Pos) {
    // Iterative deepening
    let mut depth = 1;
    let mut score;
    let start = time::Instant::now();
    loop {
        stats::new_depth();
        score = negascout(&mut p, depth, i32::MIN + 1, i32::MAX - 1);

        if *STOP.read().unwrap() {
            return;
        }

        print_info(
            TT.get(&p.key()).mv,
            depth,
            (time::Instant::now() - start).as_millis(),
            // Since eval is dependant on the color
            score.abs(),
        );

        depth += 1;
    }
}

const LMR_MOVES: usize = 2;

fn negascout(p: &mut pos::Pos, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
    if depth == 0 {
        return eval::eval(p);
    }

    let (replace_entry, mut best_mv, return_val) = parse_tt(&p.key(), depth, &mut alpha, &mut beta);
    if let Some(r) = return_val {
        return r;
    }

    // Null Move
    if depth > 3 {
        let (legal, was_ep, ep_file) = make::make_null(p);
        if legal {
            let score = -negascout(p, depth - 3, -beta, -(beta - 1));
            if score >= beta {
                make::unmake_null(p, was_ep, ep_file);
                stats::null_move_prune();
                return beta;
            }
        }
        make::unmake_null(p, was_ep, ep_file);
    }

    let mut node_type = tt::EntryType::Upper;
    let mut first_iteration = true;

    // Process PV move
    if let Some(mut m) = best_mv {
        first_iteration = false;
        make::make(p, &mut m, false);
        let score = -negascout(p, depth - 1, -beta, -alpha);
        if score > alpha {
            alpha = score;
            node_type = tt::EntryType::Exact;
        }

        if score >= beta {
            stats::beta_prune();
            make::unmake(p, &mut m);
            history::set(&m, p.clr, depth);

            if replace_entry {
                TT.set(tt::Entry::new(
                    p.key(),
                    alpha,
                    best_mv.unwrap(),
                    depth,
                    tt::EntryType::Lower,
                ));
            }
            return alpha;
        }

        make::unmake(p, &mut m);
    }

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
        let legal = make::make(p, &mut m, true);
        if !legal {
            make::unmake(p, &mut m);
            continue;
        }

        let mut score;
        if first_iteration {
            first_iteration = false;
            // Principle variation search
            // PV Node
            best_mv = Some(m);
            score = -negascout(p, depth - 1, -beta, -alpha);
        } else {
            // Null window search
            if depth > 2 && i > LMR_MOVES && do_lmr {
                // Late move reduction
                let reduced_depth = if depth < 6 { depth - 1 } else { depth / 3 };
                score = -negascout(p, reduced_depth, -alpha - 1, -alpha);
            } else {
                // Not reduced depth null window
                score = -negascout(p, depth - 1, -alpha - 1, -alpha);
            }

            if alpha < score && score < beta {
                // Unstable Node -> Dont do LMR
                if i <= LMR_MOVES {
                    do_lmr = false;
                }
                // Failed high -> Full re-search
                score = -negascout(p, depth - 1, -beta, -score);
            }
        }

        if score > alpha {
            alpha = score;
            best_mv = Some(m);
            node_type = tt::EntryType::Exact;
        }

        if score >= beta {
            // Cut Node
            stats::beta_prune();
            node_type = tt::EntryType::Lower;
            make::unmake(p, &mut m);
            history::set(&m, p.clr, depth);
            break; // Prune :)
        }

        make::unmake(p, &mut m);
    }

    // We never encountered a valid move
    if first_iteration {
        let king_pos = p.piece(Piece::King.clr(p.clr)).get_ones_single();
        if mv::mv_gen::square_not_attacked(p, king_pos, p.clr.flip()) {
            // Stalemate
            return 0;
        } else {
            // Checkmate
            return i32::MIN + 2;
        }
    }

    if replace_entry {
        TT.set(tt::Entry::new(
            p.key(),
            alpha,
            best_mv.unwrap(),
            depth,
            node_type,
        ));
    }

    alpha
}

fn parse_tt(
    key: &tt::Key,
    depth: u8,
    alpha: &mut i32,
    beta: &mut i32,
) -> (bool, Option<Mv>, Option<i32>) {
    let mut replace = false;
    let mut pv_move = None;
    let mut return_val = None;

    let entry = TT.get(key);
    if depth > entry.depth {
        replace = true;
    }

    if !entry.is_null() {
        if &entry.key != key {
            stats::tt_collision();
        } else {
            stats::tt_hit();
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
                    _ => unreachable!(),
                }

                if alpha >= beta {
                    return_val = Some(entry.score);
                }
            }
        }
    }

    (replace, pv_move, return_val)
}

fn print_info(best: Mv, depth: u8, time: u128, score: i32) {
    let info_string = format!(
        "info depth {} pv {} time {} score cp {} nodes {}",
        depth,
        best,
        time,
        score,
        stats::nodes()
    );
    println!("{}", info_string);
}

pub fn stop_search(p: &mut pos::Pos) -> Option<Mv> {
    stats::print_stats();
    *STOP.write().unwrap() = true;
    let best = TT.checked_get(&p.key());
    match best {
        None => panic!("Starting pos doesnt have a tt entry"),
        Some(e) => {
            let mut best = e.mv;
            print!("bestmove {}", best);
            make::make(p, &mut best, false);
            let ponder = TT.checked_get(&p.key());
            match ponder {
                Some(pe) => {
                    println!(" ponder {}", pe.mv);
                    return Some(pe.mv);
                }
                None => println!(),
            }
            make::unmake(p, &mut best);
        }
    }
    None
}
