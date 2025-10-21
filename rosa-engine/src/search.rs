use crate::eval::simple_eval;
use crate::make;
use crate::make::unmake;
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
    let stop = Arc::new(RwLock::new(false));

    let pclone = p.clone();
    let sclone = Arc::clone(&stop);
    thread::spawn(move || search(pclone, max_time, sclone));
    stop
}

pub fn search(mut p: pos::Pos, max_time: time::Duration, stop: Arc<RwLock<bool>>) {
    // Iterative deepening
    let mut depth = 1;
    let mut score = 0;
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
            max_time.as_millis() as u64,
            score,
            false,
        );

        depth += 1;
    }

    stats::print_tt_info();

    write_info(
        TT.get(&p.key()).mv,
        depth,
        max_time.as_millis() as u64,
        score,
        true,
    );
}
// state, depth, alpha, beta, ply from root, prev zobrist key -> eval
fn negascout(p: &mut pos::Pos, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
    // Search is done
    if depth == 0 {
        return simple_eval(p);
    }
    stats::node_count();

    let (replace_entry, mut best_mv, return_val) = parse_tt(&p.key(), depth, &mut alpha, &mut beta);
    if let Some(r) = return_val {
        return r;
    }

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
                score = -negascout(p, depth - 1, -beta, -alpha);
            }
        }

        if score > alpha {
            alpha = score;
            best_mv = Some(m);
            node_type = tt::EntryType::Exact;

            // Beta cutoff can only occur on a change of alpha
            if score >= beta {
                // Cut Node
                stats::beta_prune();
                node_type = tt::EntryType::Lower;
                make::unmake(p, &mut m);
                break; // Prune :)
            }
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
            }
        }
    }

    (replace, pv_move, return_val)
}

fn write_info(best: Mv, depth: u8, time: u64, score: i32, write_best: bool) {
    let info_string = format!(
        "info depth {} pv {} time {} score cp {} ",
        depth, best, time, score
    );
    println!("{}", info_string);
    if write_best {
        println!("bestmove {}", best)
    }
}

pub fn counting_search(p: &mut pos::Pos, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let entry = TT.get(&p.key());

    if entry.node_type == tt::EntryType::Exact && entry.key == p.key() && entry.depth == depth {
        // We found a valid entry
        return entry.score as u64;
    }

    let mut count: u64 = 0;
    let mv_iter = mv::mv_gen::gen_mvs(p);
    for mut mv in mv_iter {
        let prev_key = p.key();
        let legal = make::make(p, &mut mv);
        if !legal {
            make::unmake(p, &mut mv);
            if p.key() != prev_key {
                panic!("Key mismatch after move: {:?}", mv);
            }
            continue;
        }
        count += counting_search(p, depth - 1);
        make::unmake(p, &mut mv);
        if p.key() != prev_key {
            panic!("Key mismatch after move: {:?}", mv);
        }
    }

    TT.set(tt::Entry {
        key: (p.key()),
        score: (count as i32),
        mv: (Mv::default()),
        depth: (depth),
        node_type: (tt::EntryType::Exact),
    });

    count
}

pub fn division_search(p: &mut pos::Pos, depth: u8) {
    let mut total = 0;
    TT.resize(10000);
    for mut mv in mv::mv_gen::gen_mvs(p) {
        let legal = make::make(p, &mut mv);
        if !legal {
            unmake(p, &mut mv);
            continue;
        }
        let count = counting_search(p, depth - 1);
        make::unmake(p, &mut mv);
        total += count;
        println!("{}: {}", mv, count);
    }
    println!("Nodes searched: {total}\n");
}

pub fn debug_search(p: &mut pos::Pos, depth: u8, previous_mvs: &mut Vec<Mv>) {
    if depth == 0 {
        return;
    }

    let mv_res = std::panic::catch_unwind(|| mv::mv_gen::gen_mvs(p));
    let mv_iter;
    match mv_res {
        Ok(p) => mv_iter = p,
        Err(_e) => {
            panic!("Error in mv generation, Previous Mvs: {:?}", previous_mvs)
        }
    }
    for mut mv in mv_iter {
        let prev_key = p.key();
        let prev_pos = p.clone();
        // Ugly, but the only way to keep a list of made moves
        let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| make::make(p, &mut mv)));
        match err {
            Ok(legal) => {
                if !legal {
                    make::unmake(p, &mut mv);
                    if p.key() != prev_key {
                        panic!(
                            "Key mismatch after illegal move: {:?}\nPrevious Mvs: {:?}\nREPORT: {}",
                            mv,
                            previous_mvs,
                            pos::Pos::debug_key_mismatch(&prev_pos, p)
                        );
                    }
                    continue;
                }
            }
            Err(_e) => {
                panic!(
                    "Make Panic, Previous Mvs: {:?},\n The panic mv: {mv}",
                    previous_mvs
                );
            }
        }
        let mut clone = previous_mvs.clone();
        clone.push(mv);
        debug_search(p, depth - 1, &mut clone);
        make::unmake(p, &mut mv);
        if p.key() != prev_key {
            panic!(
                "Key mismatch after move: {:?}\nPrevious Mvs:\n{:?}, Pos before make:\n{}, Pos after unmake:\n{}\nREPORT: {}",
                mv, previous_mvs, prev_pos, p, pos::Pos::debug_key_mismatch(&prev_pos, p)
);
        }
    }
}
