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

pub fn search(p: &pos::Pos, time: u64, maxdepth: u8, tt: &mut table::TT) {
    let start = current_time();
    let time_to_search = if time != 0 { time } else { 10 * 60 * 1000 };

    // Iterative deepening
    let mut depth = 1;
    let mut score = 0;
    loop {
        debug!("Starting search at depth: {}", depth);
        if depth == maxdepth {
            break;
        }
        let searched_time = current_time() - start;
        if searched_time > time_to_search {
            break;
        }

        score = negascout(p, depth, i32::MIN + 1, i32::MAX - 1, tt);

        write_info(p, tt, depth, time, score);

        depth += 1;
    }

    debug!("Search done");
    write_info(p, tt, depth, time, score);

    let bestmove = &tt.get(&p.key).mv;
    println!("bestmove {}", bestmove.notation());
}

// state, depth, alpha, beta, ply from root, prev zobrist key -> eval
fn negascout(p: &pos::Pos, depth: u8, mut alpha: i32, mut beta: i32, tt: &mut table::TT) -> i32 {
    // Search is done
    if depth == 0 {
        return eval(p);
    }

    // Check the transposition table
    let entry = tt.get(&p.key);

    let mut pvs_move = Mv::null();
    let mut replace_entry = false;

    if entry.node_type == table::NodeType::Null {
        // The entry is unanitialized
        replace_entry = true;
    } else if entry.key != p.key {
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
            score = -negascout(&npos, depth - 1, -beta, -alpha, tt);
        } else {
            // Null window search
            score = -negascout(&npos, depth - 1, -alpha - 1, -alpha, tt);
            if alpha < score && score < beta {
                // Failed high -> Full re-search
                score = -negascout(&npos, depth - 1, -beta, -alpha, tt);
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
        tt.set(table::Entry::new(p.key, alpha, best_move, depth, node_type));
    }

    alpha
}

fn write_info(pos: &pos::Pos, tt: &table::TT, depth: u8, time: u64, score: i32) {
    log::info!("Search with depth {} concluded", depth);
    let res = tt.get(&pos.key);
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

pub fn counting_search(p: &pos::Pos, depth: u8, tt: &mut table::TT, use_tt: bool) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut count: u64 = 0;
    let mv_iter = mv::mv_gen::gen_mvs(p).filter(|mv| !mv.is_null());
    for mv in mv_iter {
        let npos = mv::mv_apply::apply(p, &mv);
        let npos = match npos {
            Some(n) => n,
            None => continue,
        };
        count += counting_search(&npos, depth - 1, tt, use_tt);
    }
    if use_tt {
        let entry = tt.get(&p.key);
        if entry.score == 0 {
        } else {
            if entry.key == p.key {
                if entry.score != count as i32 {
                    scream!("Entry found with incorect count")
                } else {
                    println!("TT hit, count: {count}");
                }
            } else {
                // hash collision
                tt.set(table::Entry {
                    key: (p.key),
                    score: (count as i32),
                    mv: (Mv::null()),
                    depth: (depth),
                    node_type: (table::NodeType::Null),
                });
            }
        }
    }
    count
}

pub fn division_search(p: &pos::Pos, depth: u8) {
    let mut total = 0;
    let mut dud = table::TT::default();
    for mv in mv::mv_gen::gen_mvs(p).filter(|mv| !mv.is_null()) {
        let npos = mv::mv_apply::apply(p, &mv);
        let npos = match npos {
            Some(p) => p,
            None => continue,
        };
        let count = counting_search(&npos, depth - 1, &mut dud, false);
        total += count;
        println!("{}: {}", mv.notation(), count);
    }
    println!("Nodes searched: {total}\n");
}
