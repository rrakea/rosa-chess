use std::cell::UnsafeCell;

use crate::clr::Clr;
use crate::mv::Mv;

// from sq * to sq * 2 colors
const TABLE_SIZE: usize = 64 * 64 * 2;

thread_local! { static HISTORY: UnsafeCell<[u8; TABLE_SIZE]> = const {UnsafeCell::new([0; TABLE_SIZE])}}

pub fn set(m: &Mv, clr: Clr, depth: u8) {
    let (from, to) = m.sq();
    let index = index(from, to, clr);
    HISTORY.with(|history| unsafe {
        let history = &mut *history.get();
        history[index] = depth * depth;
    })
}

pub fn get(m: &Mv, clr: Clr) -> u32 {
    let (from, to) = m.sq();
    let index = index(from, to, clr);
    let raw_val = HISTORY.with(|t| unsafe {
        let history = &mut *t.get();
        history[index]
    });
    raw_val as u32 / 8
}

fn index(from: u8, to: u8, clr: Clr) -> usize {
    let mut clr_bonus = 0;
    if clr.is_black() {
        clr_bonus = 64 * 64;
    }

    (from as usize * 64 + to as usize) + clr_bonus
}
