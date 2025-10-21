use crate::config;

static mut HIT: u64 = 0;
static mut COLLISION: u64 = 0;
static mut NODE_COUNT: u64 = 0;
static mut BETA_PRUNE: u64 = 0;

#[inline(always)]
pub fn tt_hit() {
    if config::TRACK_TT_INFO {
        unsafe { HIT += 1 }
    }
}

#[inline(always)]
pub fn tt_collision() {
    if config::TRACK_TT_INFO {
        unsafe { COLLISION += 1 }
    }
}

#[inline(always)]
pub fn node_count() {
    if config::TRACK_TT_INFO {
        unsafe { NODE_COUNT += 1 }
    }
}

#[inline(always)]
pub fn beta_prune() {
    if config::TRACK_TT_INFO {
        unsafe { BETA_PRUNE += 1 }
    }
}

#[inline(always)]
pub fn print_tt_info() {
    if config::TRACK_TT_INFO {
        println!("Hits: {}", unsafe { HIT});
        println!("Collisions: {}", unsafe { COLLISION });
        println!("Pos: {}", unsafe { NODE_COUNT });
        println!("Ratio: {}%", unsafe {
            HIT as f64 / NODE_COUNT as f64
        });
        println!("Beta: {}", unsafe { BETA_PRUNE });
    }
}
