//! # Multithreaded searching
//! Not ready for multithreading yet, setting up infrastructure
 
use crate::search;
use crate::stats2::{GlobalStats, SearchStats};

use rosa_lib::pos;
use rosa_lib::mv::Mv;

use std::sync::atomic;
use std::sync::mpsc;
use std::thread;

/// Used to asynchronisly stop the search function.
static STOP: atomic::AtomicBool = atomic::AtomicBool::new(false);

pub fn search_done() -> bool {
    STOP.load(atomic::Ordering::Relaxed)
}

const THREAD_COUNT: usize = 1;

/// Spawns threads and start search
/// Collects the thread reports and compiles them
pub fn thread_handler(p: &pos::Pos) {
    STOP.store(false, atomic::Ordering::Relaxed);
    let (sender, reciever) = mpsc::channel::<ThreadReport>();
    for _ in 0..THREAD_COUNT {
        let pclone = p.clone();
        let thread_sender = sender.clone();
        thread::spawn(move || {
            search::search(pclone, thread_sender);
        });
    }

    let mut global_stats = GlobalStats::new();
    let mut thread_reports: Vec<Vec<ThreadReport>> = Vec::new();

    for report in reciever.recv() {
        thread_reports[report.depth as usize].push(report);
        if thread_reports[report.depth as usize].len() == THREAD_COUNT {
            global_stats.depth_end(
                thread_reports[report.depth as usize].iter().map(|r| r.stats.clone()).collect(),
                TT.get(&p.key()).mv,
                report.score,
                report.depth as u8,
            );
        }
    }
}
pub struct ThreadReport {
    depth: u8,
    score: i32,
    pv: Option<Mv>,
    stats: SearchStats,
}


impl ThreadReport {
    pub fn new(depth: u8, score: i32, pv: Option<Mv>, stats: SearchStats) -> Self {
        ThreadReport {
            depth,
            score,
            pv,
            stats,
        }
    }
}