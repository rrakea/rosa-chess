use crate::config;
use crate::search::TT;

static mut HIT: u64 = 0;
static mut COLLISION: u64 = 0;
static mut NODE_COUNT: u64 = 0;
static mut BETA_PRUNE: u64 = 0;
static mut PREV_NODE_COUNT: u64 = 0;
static mut NULL_MOVE_PRUNE: u64 = 0;

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

pub fn nodes() -> u64 {
    unsafe { NODE_COUNT }
}

pub fn reset_node_count() {
    unsafe { NODE_COUNT = 0 }
}

#[inline(always)]
pub fn update_branching_factor() {
    if config::REPORT_STATS {
        unsafe {
            PREV_NODE_COUNT = NODE_COUNT;
        }
    }
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
        println!("Effective Branching Factor: {}", unsafe {
            NODE_COUNT as f64 / PREV_NODE_COUNT as f64
        });
        println!();

        let (valid, _null, size) = TT.usage();
        println!("TT Usage: {}%", (valid as f64 / size as f64) * 100.0);
    }
}
