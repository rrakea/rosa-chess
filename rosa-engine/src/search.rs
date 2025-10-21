use crate::eval::simple_eval;
use crate::make;
use crate::mv;
use crate::mv::mv_gen;
use crate::stats;

use rosa_lib::mv::Mv;
use rosa_lib::piece::*;
use rosa_lib::pos;
use rosa_lib::tt;

use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time;

/*
    Idea: We check if our position is in the TT at the start of a search
    -> If it is we can start our iterative deepening at that depth value
    -> Does this interfere with alpha beta pruning (If our nodes is a cut
    node? )
*/

pub static TT: tt::TT = tt::TT::new();

pub fn thread_search(p: &pos::Pos, max_time: time::Duration) -> Arc<RwLock<bool>> {
    debug_assert!(!p.is_default(), "Pos is default");
    let stop = Arc::new(RwLock::new(false));

    let pclone = p.clone();
    let sclone = Arc::clone(&stop);
    thread::spawn(move || search(pclone, max_time, sclone));
    stop
}

pub fn search(mut p: pos::Pos, max_time: time::Duration, stop: Arc<RwLock<bool>>) {
    // Iterative deepening
    let mut depth = 1;
    let mut score;
    let start = time::Instant::now();
    loop {
        if *stop.read().unwrap() {
            break;
        }

        if !max_time.is_zero() && time::Instant::now() - start >= max_time {
            break;
        }

        score = negascout(&mut p, depth, i32::MIN + 1, i32::MAX - 1);

        write_info(
            TT.get(&p.key()).mv,
            depth,
            (time::Instant::now() - start).as_millis(),
            score,
            false,
        );

        depth += 1;
    }

    stats::print_tt_info();
}

// state, depth, alpha, beta, ply from root, prev zobrist key -> eval
fn negascout(p: &mut pos::Pos, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
    if depth == 0 {
        return simple_eval(p);
    }

    let (replace_entry, mut best_mv, return_val) = parse_tt(&p.key(), depth, &mut alpha, &mut beta);
    if let Some(r) = return_val {
        return r;
    }

    stats::node_count();

    let mut node_type = tt::EntryType::Upper;
    let mut first_iteration = true;

    let iter: Box<dyn Iterator<Item = Mv>> = match best_mv {
        Some(pv_move) => Box::new(
            std::iter::once(pv_move).chain(
                mv_gen::gen_mvs(p)
                    .into_iter()
                    .filter(move |mv| *mv != pv_move),
            ),
        ),
        None => Box::new(mv_gen::gen_mvs(p).into_iter()),
    };

    for mut m in iter {
        let legal = make::make(p, &mut m);
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
            score = -negascout(p, depth - 1, -alpha - 1, -alpha);
            if alpha < score && score < beta {
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

fn write_info(best: Mv, depth: u8, time: u128, score: i32, write_best: bool) {
    let info_string = format!(
        "info depth {} pv {} time {}ms score cp {} ",
        depth, best, time, score
    );
    println!("{}", info_string);
    if write_best {
        println!("bestmove {}", best)
    }
}
