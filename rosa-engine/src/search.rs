use crate::debug;
use crate::eval::simple_eval;
use crate::make::make;
use crate::mv;
use crate::mv::mv_gen;

use rosa_lib::mv::Mv;
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

        if debug::print_tt_hits() {
            println!("Hits: {}", unsafe { HIT_COUNTER });
            println!("Collisions: {}", unsafe { COLLISION });
            println!("Null hits: {}", unsafe { NULL_HIT });
            println!("Pos: {}", unsafe { POS_COUNTER });
            println!("Ratio: {}%", unsafe {
                HIT_COUNTER as f64 / POS_COUNTER as f64
            })
        }

        if debug::print_prunes() {
            println!("Beta: {}", unsafe { BETA_PRUNE });
        }
    }

    write_info(
        TT.get(&p.key()).mv,
        depth,
        max_time.as_millis() as u64,
        score,
        true,
    );
}

static mut HIT_COUNTER: u64 = 0;
static mut COLLISION: u64 = 0;
static mut NULL_HIT: u64 = 0;
static mut POS_COUNTER: u64 = 0;

static mut BETA_PRUNE: u64 = 0;

// state, depth, alpha, beta, ply from root, prev zobrist key -> eval
fn negascout(p: &mut pos::Pos, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
    // Search is done
    if depth == 0 {
        return simple_eval(p);
    }

    if debug::print_tt_hits() {
        unsafe {
            POS_COUNTER += 1;
        }
    }

    // Check the transposition table
    let mut pv_move = Mv::default();
    let mut replace_entry = false;

    {
        let entry = TT.get(&p.key());

        // The entry is "worth" less than what we are going to write
        if depth > entry.depth {
            replace_entry = true;
        }

        // The entry is usable
        if !entry.is_null() && entry.key == p.key() {
            if debug::print_tt_hits() {
                unsafe {
                    HIT_COUNTER += 1;
                }
            }

            pv_move = entry.mv;
            // If the depth is worse we cant use the score
            if depth <= entry.depth {
                match entry.node_type {
                    tt::NodeType::Exact => {
                        return entry.score;
                    }
                    tt::NodeType::Upper => {
                        if entry.score <= alpha {
                            return entry.score;
                        } else {
                            beta = entry.score;
                        }
                    }
                    tt::NodeType::Lower => {
                        if entry.score >= beta {
                            return entry.score;
                        } else {
                            alpha = entry.score;
                        }
                    }
                    _ => (),
                }
            }
        } else if debug::print_tt_hits() {
            unsafe {
                if entry.is_null() {
                    NULL_HIT += 1;
                } else {
                    COLLISION += 1;
                }
            }
        }
    }

    let mut node_type = tt::NodeType::Upper;
    let mut first_iteration = true;

    let mv_iter = std::iter::once(pv_move).chain(
        mv_gen::gen_mvs(p)
            .into_iter()
            .filter(move |mv| *mv != pv_move),
    );

    for mut m in mv_iter {
        let legal = make(p, &mut m, true);
        if !legal {
            continue;
        }

        let mut score;
        if first_iteration {
            first_iteration = false;
            // Principle variation search
            // PV Node
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
            pv_move = m;
            node_type = tt::NodeType::Exact;

            // Beta cutoff can only occur on a change of alpha
            if score >= beta {
                // Cut Node
                if debug::print_prunes() {
                    unsafe {
                        BETA_PRUNE += 1;
                    }
                }
                node_type = tt::NodeType::Lower;
                break; // Prune :)
            }
        }
    }

    // We never encountered a valid move
    if first_iteration {
        let king_pos = p.piece(pos::KING * p.active).get_ones_single();
        if mv::mv_gen::square_not_attacked(p, king_pos, -p.active) {
            // Stalemate
            return 0;
        } else {
            // Checkmate
            return i32::MIN + 2;
        }
    }

    if replace_entry {
        //println!("Writing to TT");
        TT.set(tt::Entry::new(p.key(), alpha, pv_move, depth, node_type));
    }

    alpha
}

fn write_info(best: Mv, depth: u8, time: u64, score: i32, write_best: bool) {
    let info_string = format!(
        "info depth {} pv {} time {} score cp {} ",
        depth,
        best.notation(),
        time,
        score
    );
    println!("{}", info_string);
    if write_best {
        println!("bestmove {}", best.notation())
    }
}

pub fn counting_search(p: &mut pos::Pos, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let entry = TT.get(&p.key());

    if entry.node_type == tt::NodeType::Exact && entry.key == p.key() && entry.depth == depth {
        // We found a valid entry
        return entry.score as u64;
    }

    let mut count: u64 = 0;
    let mv_iter = mv::mv_gen::gen_mvs(p);
    for mut mv in mv_iter {
        let legal = make(p, &mut mv, true);
        if !legal {
            continue;
        }
        count += counting_search(p, depth - 1);
    }

    TT.set(tt::Entry {
        key: (p.key()),
        score: (count as i32),
        mv: (Mv::default()),
        depth: (depth),
        node_type: (tt::NodeType::Exact),
    });

    count
}

pub fn division_search(p: &mut pos::Pos, depth: u8) {
    let mut total = 0;
    TT.resize(10000);
    for mut mv in mv::mv_gen::gen_mvs(p){
        make(p, &mut mv, true);
        let count = counting_search(p, depth - 1);
        total += count;
        println!("{}: {}", mv.notation(), count);
    }
    println!("Nodes searched: {total}\n");
}
