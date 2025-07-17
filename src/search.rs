use crate::eval::eval;
use crate::mv;
use crate::mv::mv::Mv;
use crate::pos;
use crate::table;
use std::time;

/*
    Idea: We check if our position is in the TT at the start of a search
    -> If it is we can start our iterative deepening at that depth value
    -> Does this interfere with alpha beta pruning (If our nodes is a cut
    node? )
*/

// This will stop working in 292 billion years :(
static mut START: u64 = 0;
static mut TIME_TO_SEARCH: u64 = 0;

pub fn search(
    p: &pos::Pos,
    time: u64,
    maxdepth: u8,
    key: &mut table::Key,
    tt: &mut table::TT,
) -> (u8, u64) {
    log::info!("Starting search");
    // Safe since none of the threads have started searching yet
    // Wont be mutated till the next move is made
    unsafe {
        START = time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        TIME_TO_SEARCH = time;
    }

    // Iterative deepening
    let mut depth = 1;
    let mut score = 0;
    loop {
        if depth == maxdepth {
            break;
        }
        let searched_time = current_time() - unsafe { START };
        if searched_time > unsafe { TIME_TO_SEARCH } {
            break;
        }

        score = negascout(p, depth, i32::MIN, i32::MAX, tt, key);

        write_info(tt, &key, depth, time, score);

        depth += 1;
    }

    log::info!("Search done");
    write_info(tt, &key, depth, time, score);

    (depth + 1, current_time() - unsafe { START })
}

// state, depth, alpha, beta, ply from root, prev zobrist key -> eval
fn negascout(
    p: &pos::Pos,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    tt: &mut table::TT,
    key: &mut table::Key,
) -> i32 {
    // Search is done
    if depth == 0 {
        return eval(p);
    }

    // Check the transposition table
    let entry = tt.get(&key);

    let mut tt_hash_move = Mv::null();
    let mut replace_entry = false;

    if entry.node_type == table::NodeType::Null {
        // The entry is unanitialized
        replace_entry = true;
    } else if entry.key != *key {
        // The entry is not the same pos as outs
        // Dont replace if the entry is higher in the tree
        if entry.depth > depth {
            replace_entry = true;
        }
    } else {
        tt_hash_move = entry.mv;
        // The entry is usable
        if entry.depth < depth {
            // Cant trust the eval; Still use the best move
            replace_entry = true;
        } else {
            match entry.node_type {
                table::NodeType::Exact => {
                    // The entries analysis is better than ours
                    return entry.score;
                }
                table::NodeType::Upper => {
                    if entry.score >= beta {
                        return entry.score;
                    } else {
                        beta = entry.score;
                    }
                }
                table::NodeType::Lower => {
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
    let mut node_type = table::NodeType::Upper;

    // Iterator
    let gen_mvs = mv::mv_gen::gen_mvs(p, tt_hash_move);
    let ordered_mvs = mv::mv_order::order_mvs(gen_mvs);
    
    let mut legal_move_exists = true;
    for (i, m) in ordered_mvs.enumerate() {
        let outcome = mv::mv_apply::apply(p, &m, key);
        let outcome = match outcome {
            Some(o) => o,
            // Impossible move
            None => continue,
        };
        legal_move_exists = true;

        let mut score;
        if i == 0 {
            // Principle variation search
            // PV Node
            score = -negascout(&outcome, depth - 1, -beta, -alpha, tt, key);
        } else {
            // Null window search
            score = -negascout(&outcome, depth - 1, -alpha - 1, -alpha, tt, key);
            if alpha < score && score < beta {
                // Failed high -> Full re-search
                score = -negascout(&outcome, depth - 1, -beta, -alpha, tt, key);
            }
        }
        if score > alpha {
            alpha = score;
            best_move = m;
            node_type = table::NodeType::Exact;

            // Beta cutoff can only occur on a change of alpha
            if alpha >= beta {
                // Cut Node
                node_type = table::NodeType::Lower;
                break; // Prune :)
            }
        }
    }

    if !legal_move_exists {
        let king_pos = p.piece(pos::KING * p.active).get_ones_single();
        if mv::mv_gen::square_attacked(p, king_pos, p.active * -1) {
            // Checkmate
            return i32::MIN;
        } else {
            // Stalemate
            return 0;
        }
    }

    if replace_entry {
        tt.set(table::Entry::new(
            key.clone(),
            alpha,
            best_move,
            depth,
            node_type,
        ));
    }

    alpha
}

fn write_info(tt: &table::TT, start_key: &table::Key, depth: u8, time: u64, score: i32) {
    log::info!("Search with depth {} concluded", depth);
    let res = tt.get(&start_key);
    let info_string = format!(
        "info depth {} time {} pv {} score cp {} ",
        depth,
        time,
        res.mv.notation(),
        score
    );
    log::info!("Command send: {}", info_string);
    println!("{}", info_string);
}

fn current_time() -> u64 {
    time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
