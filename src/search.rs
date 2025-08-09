use crate::eval::eval;
use crate::mv;
use crate::mv::mv::Mv;
use crate::pos;
use crate::tt;

use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

/*
    Idea: We check if our position is in the TT at the start of a search
    -> If it is we can start our iterative deepening at that depth value
    -> Does this interfere with alpha beta pruning (If our nodes is a cut
    node? )
*/

pub static TT: tt::TT = tt::TT::new();

pub fn thread_search(p: &pos::Pos) -> Arc<RwLock<bool>> {
    let stop = Arc::new(RwLock::new(false));

    let pclone = p.clone();
    let sclone = Arc::clone(&stop);
    thread::spawn(move || search(pclone, sclone));
    stop
}

pub fn search(p: pos::Pos, stop: Arc<RwLock<bool>>) {
    // Iterative deepening
    let mut depth = 1;
    let mut score = 0;
    loop {
        log::info!("Starting search at depth: {}", depth);

        if *stop.read().unwrap() {
            break;
        }

        score = negascout(&p, depth, i32::MIN + 1, i32::MAX - 1);

        write_info(TT.get(&p.key()).mv, depth, score, false);

        depth += 1;
    }

    debug!("Search done");
    write_info(TT.get(&p.key()).mv, depth, score, true);
}

// state, depth, alpha, beta, ply from root, prev zobrist key -> eval
fn negascout(p: &pos::Pos, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
    // Search is done
    if depth == 0 {
        return eval(p);
    }

    // Check the transposition table
    let entry = TT.get(&p.key());

    let mut pvs_move = Mv::null();
    let mut replace_entry = false;

    if entry.node_type == tt::NodeType::Null {
        // The entry is unanitialized
        replace_entry = true;
    } else if entry.key != p.key() {
        // The entry is not the same pos as outs
        // Dont replace if the entry is higher in the tree
        if entry.depth > depth {
            replace_entry = true;
        }
    } else {
        pvs_move = entry.mv;
        // The entry is usable
        if entry.depth < depth {
            // Cant trust the eval; Still use the best move
            replace_entry = true;
        } else {
            match entry.node_type {
                tt::NodeType::Exact => {
                    // The entries analysis is better than ours
                    return entry.score;
                }
                tt::NodeType::Upper => {
                    if entry.score >= beta {
                        return entry.score;
                    } else {
                        beta = entry.score;
                    }
                }
                tt::NodeType::Lower => {
                    if entry.score <= alpha {
                        return entry.score;
                    } else {
                        alpha = entry.score;
                    }
                }
                _ => {} // Unreachable
            }
        }
    }

    let mut best_move = Mv::null();
    let mut node_type = tt::NodeType::Upper;

    // Iterator
    let gen_mvs = mv::mv_gen::gen_mvs(p).filter(|mv| !mv.is_null());
    let ordered_mvs = mv::mv_order::order_mvs(gen_mvs).filter(|mv| *mv != pvs_move);
    let mv_iter = std::iter::once(pvs_move)
        .chain(ordered_mvs)
        .filter(|mv| !mv.is_null());

    let mut legal_move_exists = true;
    for (i, m) in mv_iter.enumerate() {
        let outcome = mv::mv_apply::apply(p, &m);
        let npos = match outcome {
            Some(o) => o,
            // Impossible move
            None => continue,
        };
        //debug!("Searching move: {} at depth: {}", m.prittify(), depth);
        //debug!("PVS move: {}", pvs_move.prittify());
        legal_move_exists = true;

        let mut score;
        if i == 0 {
            // Principle variation search
            // PV Node
            score = -negascout(&npos, depth - 1, -beta, -alpha);
        } else {
            // Null window search
            score = -negascout(&npos, depth - 1, -alpha - 1, -alpha);
            if alpha < score && score < beta {
                // Failed high -> Full re-search
                score = -negascout(&npos, depth - 1, -beta, -alpha);
            }
        }
        if score > alpha {
            alpha = score;
            best_move = m;
            node_type = tt::NodeType::Exact;

            // Beta cutoff can only occur on a change of alpha
            if alpha >= beta {
                // Cut Node
                node_type = tt::NodeType::Lower;
                break; // Prune :)
            }
        }
    }

    if !legal_move_exists {
        debug!("Found checkmate at depth: {depth}");
        let king_pos = p.piece(pos::KING * p.active).get_ones_single();
        if mv::mv_gen::square_not_attacked(p, king_pos, -p.active) {
            // Checkmate
            return i32::MIN + 1;
        } else {
            // Stalemate
            return 0;
        }
    }

    if replace_entry {
        TT.set(tt::Entry::new(p.key(), alpha, best_move, depth, node_type));
    }

    alpha
}

fn write_info(best: Mv, depth: u8, score: i32, write_best: bool) {
    log::info!("Search with depth {} concluded", depth);
    let info_string = format!(
        "info depth {} pv {} score cp {} ",
        depth,
        best.notation(),
        score
    );
    log::info!("Command send: {}", info_string);
    println!("{}", info_string);
    if write_best {
        println!("bestmove {}", best.notation())
    }
}

pub fn counting_search(p: &pos::Pos, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let entry = TT.get(&p.key());

    if entry.node_type == tt::NodeType::Exact && entry.key == p.key() && entry.depth == depth {
        // We found a valid entry
        return entry.score as u64;
    }

    let mut count: u64 = 0;
    let mv_iter = mv::mv_gen::gen_mvs(p).filter(|mv| !mv.is_null());
    for mv in mv_iter {
        let npos = mv::mv_apply::apply(p, &mv);
        let npos = match npos {
            Some(n) => n,
            None => continue,
        };
        count += counting_search(&npos, depth - 1);
    }

    TT.set(tt::Entry {
        key: (p.key()),
        score: (count as i32),
        mv: (Mv::null()),
        depth: (depth),
        node_type: (tt::NodeType::Exact),
    });

    count
}

pub fn division_search(p: &pos::Pos, depth: u8) {
    let mut total = 0;
    TT.resize(10000);
    for mv in mv::mv_gen::gen_mvs(p).filter(|mv| !mv.is_null()) {
        let npos = mv::mv_apply::apply(p, &mv);
        let npos = match npos {
            Some(p) => p,
            None => continue,
        };
        let count = counting_search(&npos, depth - 1);
        total += count;
        println!("{}: {}", mv.notation(), count);
    }
    println!("Nodes searched: {total}\n");
}
