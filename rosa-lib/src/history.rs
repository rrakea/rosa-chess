//! ## History Heuristic
//! The history heuristic evaluates moves based on how often a move was previously evaluated.
//! The data is saved in a global tables, indexed by: from square x to square x color.
//! The formula is again different for every engine, rosa currently uses: prev history + depth^2

use std::cell::UnsafeCell;

use crate::clr::Clr;
use crate::mv::Mv;

// from sq * to sq * 2 colors
const TABLE_SIZE: usize = 64 * 64 * 2;
const MAX_HISTORY: u16 = u16::pow(2, 14);

thread_local! { static HISTORY: UnsafeCell<[u16; TABLE_SIZE]> = const {UnsafeCell::new([0; TABLE_SIZE])}}

pub fn set(m: &Mv, clr: Clr, depth: u8) {
    let (from, to) = m.sq();
    let index = index(from, to, clr);
    HISTORY.with(|history| unsafe {
        let history = &mut *history.get();
        let depth = depth as u16;
        history[index] = u16::clamp(history[index] + depth * depth, 0, MAX_HISTORY - 1);
    })
}

pub fn get(m: &Mv, clr: Clr) -> u32 {
    let (from, to) = m.sq();
    let index = index(from, to, clr);
    let raw_val = HISTORY.with(|t| unsafe {
        let history = &mut *t.get();
        history[index]
    });

    if raw_val == 0 {
        return 0;
    }

    // This needs to map into 5 bit -> Linear scaling
    let res = (raw_val as u32 * 31) / MAX_HISTORY as u32;
    debug_assert!(res < 32);
    res
}

fn index(from: u8, to: u8, clr: Clr) -> usize {
    let mut clr_bonus = 0;
    if clr.is_black() {
        clr_bonus = 64 * 64;
    }

    (from as usize * 64 + to as usize) + clr_bonus
}
