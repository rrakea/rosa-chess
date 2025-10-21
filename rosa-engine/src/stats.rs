use crate::config;
use crate::search::TT;

static mut HIT: u64 = 0;
static mut COLLISION: u64 = 0;
static mut NODE_COUNT: u64 = 0;
static mut BETA_PRUNE: u64 = 0;

#[inline(always)]
pub fn tt_hit() {
    if config::REPORT_STATS {
        unsafe { HIT += 1 }
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
    if config::REPORT_STATS {
        unsafe { NODE_COUNT += 1 }
    }
}

#[inline(always)]
pub fn beta_prune() {
    if config::REPORT_STATS {
        unsafe { BETA_PRUNE += 1 }
    }
}

#[inline(always)]
pub fn print_tt_info() {
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
        println!();

        let (valid, null, size) = TT.usage();
        println!("TT Usage:\nFilled: {valid}, Null: {null}, Total: {size}");
    }
}
