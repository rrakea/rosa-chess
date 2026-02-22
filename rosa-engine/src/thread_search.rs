//! # Multithreaded searching
//! Not ready for multithreading yet, setting up infrastructure
//! Threading Setup:
//! One thread blocks on stdin & timeout, one blocks on pulling from the search reports
//! Rest search

use crate::make;
use crate::search_helper;

use crossbeam::channel;
use rosa_lib::mv::Mv;
use rosa_lib::pos;

use std::sync::Arc;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;

const THREAD_COUNT: usize = 1;

pub fn start_thread_search(p: &pos::Pos) -> (channel::Receiver<Option<Mv>>, Stop) {
    let (tx, rx) = channel::unbounded();
    let p = p.clone();
    let stop = Stop::new();
    let stop_c = stop.clone();
    thread::spawn(|| thread_handler(p, tx, stop_c));
    (rx, stop)
}

/// Spawns threads and start search
/// Collects the thread reports and compiles them
fn thread_handler(p: pos::Pos, tx: channel::Sender<Option<Mv>>, stop: Stop) {
    let start_time = std::time::Instant::now();
    let (sender, reciever) = mpsc::channel::<ThreadReport>();
    for _ in 0..THREAD_COUNT {
        let pclone = p.clone();
        let thread_sender = sender.clone();
        let stop_clone = stop.clone();
        thread::spawn(move || {
            search_helper::search(pclone, thread_sender, stop_clone);
        });
    }
    // So we properly end the while loop
    drop(sender);
    drop(stop);

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

    let report = thread_reports.last().unwrap().first().unwrap();
    let mut pv = report.pv;

    match report.ponder {
        Some(pon) => {
            println!("bestmove {} ponder {}", pv, pon);
            tx.send(Some(pon)).unwrap();
        }
        None => {
            // This can only happen if
            // a) Search times out before any position has been searched to depth 2
            // -> Quit unlikely
            // b) We have just played a checkmating move
            // -> The resulting position has no legal moves
            let mut p_after_move = p.clone();
            let (_, guard) = make::make(&mut p_after_move, &mut pv, false);
            // SAFETY: Debug code working on a clone
            unsafe {
                guard.verified_drop();
            }
            println!("bestmove {}", pv);
            tx.send(None).unwrap();
        }
    }
}

fn print_info(
    pv: Mv, score: i32, depth: u8, nodes: u64, tt_hits: u64, start_time: std::time::Instant,
) {
    let finish_time = std::time::Instant::now();
    println!(
        "info depth {} pv {} time {} score cp {} nodes {}, nps {}, tbhits {}",
        depth,
        pv,
        finish_time.duration_since(start_time).as_millis(),
        score,
        nodes,
        (nodes / finish_time.duration_since(start_time).as_millis().max(1) as u64) * 1000,
        tt_hits,
    )
}

#[derive(Clone)]
pub struct ThreadReport {
    depth: u8,
    score: i32,
    pv: Mv,
    ponder: Option<Mv>,
    stats: SearchStats,
}

impl ThreadReport {
    pub fn new(depth: u8, score: i32, pv: Mv, ponder: Option<Mv>, stats: SearchStats) -> Self {
        ThreadReport {
            depth,
            score,
            pv,
            ponder,
            stats,
        }
    }
}

#[derive(Clone)]
pub struct SearchStats {
    pub depth: u8,
    nodes: u64,
    tt_hits: u64,
    timeout_nodes: u64,
}

impl SearchStats {
    pub fn new(depth: u8) -> Self {
        SearchStats {
            depth,
            nodes: 0,
            timeout_nodes: 0,
            tt_hits: 0,
        }
    }

    pub fn node(&mut self) {
        self.nodes += 1;
        self.timeout_nodes += 1;
    }

    pub fn check_for_timeout(&mut self) -> bool {
        if self.timeout_nodes >= 4098 {
            self.timeout_nodes = 0;
            return true;
        }
        false
    }

    pub fn tt_hit(&mut self) {
        self.tt_hits += 1;
    }
}

pub struct Stop(Arc<AtomicBool>);

impl Stop {
    pub fn new() -> Self {
        Stop(Arc::new(AtomicBool::new(false)))
    }

    pub fn stop_search(&mut self) {
        self.0.store(true, atomic::Ordering::Relaxed);
    }

    pub fn clone(&self) -> Self {
        Stop(Arc::clone(&self.0))
    }

    pub fn is_done(&self) -> bool {
        self.0.load(atomic::Ordering::Relaxed)
    }
}
