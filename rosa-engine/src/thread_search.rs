//! # Multithreaded searching
//! Not ready for multithreading yet, setting up infrastructure
//! Threading Setup:
//! One thread blocks on stdin & timeout, one blocks on pulling from the search reports
//! Rest search

use crate::make;
use crate::search;

use crossbeam::channel;
use rosa_lib::mv::Mv;
use rosa_lib::pos;

use std::sync::atomic;
use std::sync::mpsc;
use std::thread;

/// Used to asynchronisly stop the search function.
static STOP: atomic::AtomicBool = atomic::AtomicBool::new(false);

pub fn search_done() -> bool {
    STOP.load(atomic::Ordering::Relaxed)
}

pub fn stop_search() {
    STOP.store(true, atomic::Ordering::Relaxed);
}

const THREAD_COUNT: usize = 1;

pub fn start_thread_search(p: &pos::Pos) -> channel::Receiver<Mv> {
    let (tx, rx) = channel::bounded::<Mv>(THREAD_COUNT);
    let p = p.clone();
    thread::spawn(|| thread_handler(p, tx));
    rx
}

/// Spawns threads and start search
/// Collects the thread reports and compiles them
fn thread_handler(p: pos::Pos, tx: channel::Sender<Mv>) {
    let start_time = std::time::Instant::now();
    STOP.store(false, atomic::Ordering::Relaxed);
    let (sender, reciever) = mpsc::channel::<ThreadReport>();
    for _ in 0..THREAD_COUNT {
        let pclone = p.clone();
        let thread_sender = sender.clone();
        thread::spawn(move || {
            search::search(pclone, thread_sender);
        });
    }
    // So we properly end the while loop
    drop(sender);

    let mut total_nodes = 0;
    let mut thread_reports: Vec<Vec<ThreadReport>> = Vec::new();

    while let Ok(report) = reciever.recv() {
        let depth = report.depth as usize;
        while depth >= thread_reports.len() {
            thread_reports.push(Vec::new());
        }
        thread_reports[depth].push(report);
        if thread_reports[depth].len() == THREAD_COUNT {
            // All threads have reported for this depth
            let mut best_score = i32::MIN;
            let mut best_pv = thread_reports[depth][0].pv;
            let mut tt_hits = 0;

            for report in &thread_reports[depth] {
                if report.score > best_score {
                    best_score = report.score;
                    best_pv = report.pv;
                }
                total_nodes += report.stats.nodes;
                tt_hits += report.stats.tt_hits;
            }

            print_info(
                best_pv,
                best_score,
                depth as u8,
                total_nodes,
                tt_hits,
                start_time,
            );
        }
    }

    let mut pv;
    let ponder;
    // You might think that the position might get overwritten, but the root node will always write to TT at the very end
    // Same for ponder (except root key != ponder key but thats unlikely if our hashing is any good)
    // We dont bounce up the moves in the search to save mem & simplify logic
    match search::TT.get(p.key()) {
        Some(e) => {
            pv = e.mv;
            let mut pclone = p.clone();
            let (_, guard) = make::make(&mut pclone, &mut pv, false);
            // Safety: Pos gets dropped after this
            unsafe {
                guard.verified_drop();
            }
            match search::TT.get(pclone.key()) {
                Some(e) => {
                    ponder = e.mv;
                }
                None => {
                    panic!("Ponder position not in TT");
                }
            }
        }
        None => {
            panic!("Root Position not in TT");
        }
    }
    print_best_move(pv);
    tx.send(ponder).unwrap();
}

fn print_info(
    pv: Mv,
    score: i32,
    depth: u8,
    nodes: u64,
    tt_hits: u64,
    start_time: std::time::Instant,
) {
    let finish_time = std::time::Instant::now();
    println!(
        "info depth {} pv {} time {} score cp {} nodes {}, nps {}, tbhits {}",
        depth,
        pv,
        finish_time.duration_since(start_time).as_millis(),
        score,
        nodes,
        nodes / finish_time.duration_since(start_time).as_secs().max(1),
        tt_hits,
    )
}

fn print_best_move(pv: Mv) {
    println!("bestmove {}", pv);
}

pub struct ThreadReport {
    depth: u8,
    score: i32,
    pv: Mv,
    stats: SearchStats,
}

impl ThreadReport {
    pub fn new(depth: u8, score: i32, pv: Mv, stats: SearchStats) -> Self {
        ThreadReport {
            depth,
            score,
            pv,
            stats,
        }
    }
}

pub struct SearchStats {
    pub depth: u8,
    nodes: u64,
    tt_hits: u64,
}

impl SearchStats {
    pub fn new(depth: u8) -> Self {
        SearchStats {
            depth,
            nodes: 0,
            tt_hits: 0,
        }
    }

    pub fn node(&mut self) {
        self.nodes += 1;
    }

    pub fn tt_hit(&mut self) {
        self.tt_hits += 1;
    }
}
