use std::sync::RwLock;

use crate::config;
use crate::search::TT;

static mut HIT: u64 = 0;
static mut COLLISION: u64 = 0;
static mut BETA_PRUNE: u64 = 0;
static mut NULL_MOVE_PRUNE: u64 = 0;

static mut NODE_COUNT: u64 = 0;
static NODES: RwLock<Vec<u64>> = RwLock::new(Vec::new());

#[inline(always)]
pub fn tt_hit() {
    if config::REPORT_STATS {
        unsafe { HIT += 1 }
    }
}

#[inline(always)]
pub fn null_move_prune() {
    if config::REPORT_STATS {
        unsafe { NULL_MOVE_PRUNE += 1 }
    }
}

#[inline(always)]
pub fn reset_node_count() {
    unsafe {
        NODE_COUNT = 0;
    }

    let mut n = NODES.write().unwrap();
    *n = Vec::new();
}

#[inline(always)]
pub fn tt_collision() {
    if config::REPORT_STATS {
        unsafe { COLLISION += 1 }
    }
}

#[inline(always)]
pub fn node_count() {
    unsafe { NODE_COUNT += 1 }
}

#[inline(always)]
pub fn beta_prune() {
    if config::REPORT_STATS {
        unsafe { BETA_PRUNE += 1 }
    }
}

#[inline(always)]
pub fn new_depth() {
    if config::REPORT_STATS {
        unsafe {
            (*NODES.write().unwrap()).push(NODE_COUNT);
            NODE_COUNT = 0;
        }
    }
}

#[inline(always)]
pub fn nodes() -> u64 {
    return *(*NODES.write().unwrap()).last().unwrap();
}

#[inline(always)]
pub fn print_stats() {
    if config::REPORT_STATS {
        println!();
        println!("STATS:");
        println!("Hits: {}", unsafe { HIT });
        println!("Collisions: {}", unsafe { COLLISION });
        println!("Nodes: {}", unsafe { NODE_COUNT });
        println!("TT Hit ratio: {}%", unsafe {
            (HIT as f64 / NODE_COUNT as f64) * 100.0
        });
        println!("Beta Prunes: {}", unsafe { BETA_PRUNE });
        println!("Null Move Prunes: {}", unsafe { NULL_MOVE_PRUNE });
        let nodes = NODES.read().unwrap();
        println!(
            "Effective Branching Factor: {}",
            nodes[nodes.len() - 2] as f64 / nodes[nodes.len() - 3] as f64
        );
        println!("Nodes: {:?}", nodes);
        println!();

        let (valid, _null, size) = TT.usage();
        println!("TT Usage: {}%", (valid as f64 / size as f64) * 100.0);
    }
}
