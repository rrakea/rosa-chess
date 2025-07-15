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

        score = negascout(p, depth, i32::MIN, i32::MAX, 0, tt, key);

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
    mut a: i32,
    b: i32,
    ply: u8,
    tt: &mut table::TT,
    key: &mut table::Key,
) -> i32 {
    // Search is done
    if depth == 0 {
        return eval(p);
    }

    // Check the transposition table
    let entry = tt.get(&key);
    let trust_best = false;
    let replace_entry = false;

    if entry.key.is_null() {
        replace_entry = true
    } else if entry.key.val() == key.val() {
        // Transposition hit
        if entry.depth > depth {
            // The analysis is better than ours
            return entry.score;
        } else {
            // We want to analyze again, but the position is the same
            // -> We try the best move first
            trust_best = true;
            replace_entry = true;
        }
    } else {
        // The entry is from a different position
        if entry.depth < depth {
            replace_entry = true;
        }
    }

    let mut best_score = i32::MIN;
    let mut best_move = Mv::null();

    // Iterator
    let move_gen = mv::mv_gen::mv_gen(p, entry.mv, trust_best);

    let legal_move_exists = true;
    for (i, m) in move_gen.enumerate() {
        let outcome = mv::mv_apply::apply(p, &m, key);
        let outcome = match outcome {
            Some(o) => o,
            // Impossible move
            None => continue,
        };
        legal_move_exists = true;

        let mut score;
        if i == 1 && trust_best {
            score = -negascout(&outcome, depth - 1, -b, -a, ply + 1, tt, key);
        } else {
            // Null window search
            score = -negascout(&outcome, depth - 1, -a - 1, -a, ply + 1, tt, key);
            // You have to do this, since you cant do a "-" before the tupel
            if a < score && score < b {
                // Failed high -> Full re-search
                score = -negascout(&outcome, depth - 1, -b, -a, ply + 1, tt, key);
            }
        }
        a = i32::max(a, score);
        if score > best_score {
            best_score = score;
            best_move = m;
        }
        if a >= b {
            break; // Prune :)
        }
    }

    if !legal_move_exists {
        // TODO
    }

    let node_type = table::NodeType::All;
    tt.set(table::Entry::new(
        key, best_score, best_move, depth, node_type,
    ));

    best_score
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
