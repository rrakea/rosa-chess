//! # Variations on the search functions
//! Used in debugging. Extremly slow but very thorough
use rosa_engine::make;
use rosa_engine::mv;
use rosa_engine::search::TT;

use rosa_lib::mv::Mv;
use rosa_lib::pos;
use rosa_lib::tt;

pub fn counting_search(p: &mut pos::Pos, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let entry = TT.get(p.key());

    if let Some(e) = entry
        && e.key == p.key()
        && e.depth == depth
    {
        // We found a valid entry
        return e.score as u64;
    }

    let mut count: u64 = 0;

    let iter = mv::mv_gen::gen_mvs_stages(p, true)
        .into_iter()
        .inspect(|m| assert!(m.is_cap()))
        .chain(
            mv::mv_gen::gen_mvs_stages(p, false)
                .into_iter()
                .inspect(|m| assert!(!m.is_cap())),
        );

    for mut mv in iter {
        let prev_key = p.key();
        let (legal, guard) = make::make(p, &mut mv, true);
        if legal == make::Legal::ILLEGAL {
            make::unmake(p, mv, guard);
            if p.key() != prev_key {
                panic!("Key mismatch after move: {:?}", mv);
            }
            continue;
        }
        count += counting_search(p, depth - 1);
        make::unmake(p, mv, guard);
        if p.key() != prev_key {
            panic!("Key mismatch after move: {:?}", mv);
        }
    }

    TT.set(tt::Entry {
        key: (p.key()),
        score: (count as i32),
        mv: (Mv::null()),
        depth: (depth),
        node_type: (tt::EntryType::Exact),
    });

    count
}

pub fn _division_search(p: &mut pos::Pos, depth: u8) {
    let mut total = 0;
    TT.resize(10000);
    for mut mv in mv::mv_gen::gen_mvs(p) {
        let (legal, guard) = make::make(p, &mut mv, true);
        if legal == make::Legal::ILLEGAL {
            make::unmake(p, mv, guard);
            continue;
        }
        let count = counting_search(p, depth - 1);
        make::unmake(p, mv, guard);
        total += count;
        println!("{}: {}", mv, count);
    }
    println!("Nodes searched: {total}\n");
}

pub fn debug_search(p: &mut pos::Pos, depth: u8, previous_mvs: &mut Vec<Mv>) {
    if depth == 0 {
        return;
    }

    if depth > 2 {
        let prev_key = p.key();
        let prev_pos = p.clone();
        let (legal, was_ep, guard) = make::make_null(p);
        if legal == make::Legal::LEGAL {
            debug_search(p, depth - 1, previous_mvs);
        }
        make::unmake_null(p, was_ep, guard);
        if p.key() != prev_key {
            panic!(
                "Null move key mismatch, Report: {}",
                pos::Pos::debug_key_mismatch(&prev_pos, p)
            );
        }
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
        let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            make::make(p, &mut mv, true)
        }));
        let guard;
        match err {
            Ok((legal, ok_guard)) => {
                if legal == make::Legal::ILLEGAL {
                    make::unmake(p, mv, ok_guard);
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
                guard = ok_guard;
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
        make::unmake(p, mv, guard);
        if p.key() != prev_key {
            panic!(
                "Key mismatch after move: {:?}\nPrevious Mvs:\n{:?}, Pos before make:\n{}, Pos after unmake:\n{}\nREPORT: {}",
                mv,
                previous_mvs,
                prev_pos,
                p,
                pos::Pos::debug_key_mismatch(&prev_pos, p)
            );
        }
    }
}
