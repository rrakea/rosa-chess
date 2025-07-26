use crate::mv::constants;
use crate::mv::magic_init;
use crate::util;
use crate::board;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rayon::prelude::*;

const MAGIC_TRIES: u64 = 100_000_000;

const MAX_BLOCKER_ROOK: usize = usize::pow(2, 12);
const MAX_BLOCKER_BISHOP: usize = usize::pow(2, 13);

// The tupel saves the magic (u64) and the shift (u32)
static mut ROOK_MAGIC: [(u64, u8); 64] = [(0, 0); 64];
static mut BISHOP_MAGIC: [(u64, u8); 64] = [(0, 0); 64];

static mut ROOK_MOVE_WITH_BLOCKERS: [[u64; MAX_BLOCKER_ROOK]; 64] = [[0; MAX_BLOCKER_ROOK]; 64];
static mut BISHOP_MOVE_WROOK_MOVE_WITH_BLOCKERITH_BLOCKERS: [[u64; MAX_BLOCKER_BISHOP]; 64] =
    [[0; MAX_BLOCKER_BISHOP]; 64];

static mut ROOK_BLOCKERS: [[u64; MAX_BLOCKER_ROOK]; 64] = [[0; MAX_BLOCKER_ROOK]; 64];
static mut BISHOP_BLOCKERS: [[u64; MAX_BLOCKER_BISHOP]; 64] = [[0; MAX_BLOCKER_BISHOP]; 64];

pub fn gen_magics() {
    log::info!("Generating magics");
    magic_init::init_premasks();
    init_movemasks();
    (0..64).into_par_iter().for_each(|sq| {
        let mut rng = Pcg64::from_os_rng();
        log::info!("Current sq: {sq}");
        let move_mask = unsafe { constants::ROOK_PREMASKS[sq] };
        // I have never gotten a shift value below 10
        'shifts: for shift in [12, 11, 10, 9] {
            // The corners are probably not possible with under 12
            if ![0, 7, 56, 63].contains(&sq) && shift == 12 {
                continue 'shifts;
            }
            // Huge number :)
            'magics: for i in 0..MAGIC_TRIES {
                // 2 ^ shift is the max amount we can reach
                // since we truncate our index to that value
                let mut move_lookup = vec![0u64; usize::pow(2, shift as u32)];

                // Using a lot of random numbers and anding them together tends to create a lower number
                // This tends to create better magics
                let magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();

                'blockers: for (blocker_index, blocker) in
                    unsafe { ROOK_BLOCKERS[sq] }.iter().enumerate()
                {
                    // If we have reached the end of the ROOK_BLOCKERS array
                    if *blocker == 0 && blocker_index > 0 {
                        break 'blockers;
                    }

                    let blocker_mask = move_mask & *blocker;
                    let index = (blocker_mask * magic) >> (64 - shift);
                    let move_with_blocker = unsafe { ROOK_MOVE_WITH_BLOCKERS[sq][blocker_index] };
                    if move_lookup[index as usize] == 0 {
                        move_lookup[index as usize] = move_with_blocker;
                    } else {
                        // There is something already at that index
                        if move_lookup[index as usize] == move_with_blocker {
                            continue 'blockers;
                        } else {
                            continue 'magics;
                        }
                    }
                }

                // We have gone through all of the blockers and all works
                // -> The magic works!
                unsafe {
                    ROOK_MAGIC[sq] = (magic, shift);
                };
                continue 'shifts;
            }
            // If we havent update the magic this shift value
            // We assume there is no magic possible at this/ a smaller shift value
            if unsafe { ROOK_MAGIC[sq].1 } != shift {
                break 'shifts;
            }
        }
    });
    for (sq, magic) in unsafe { ROOK_MAGIC }.iter().enumerate() {
        println!("Sq: {sq}, Magic: {:#018x},", magic.0);
    }

    for (sq, magic) in unsafe { ROOK_MAGIC }.iter().enumerate() {
        println!("Sq: {sq}, Shift: {},", magic.1);
    }
}

fn init_movemasks() {
    // Rook movemasks
    for sq in 0..64 {
        // let mut premask = unsafe { ROOK_PREMASK[sq] };
        let trunc_premask = unsafe { constants::ROOK_PREMASKS_TRUNC[sq] };

        let mut blocker_index = 0;
        let mut last_iteration = false;
        loop {
            // Calculate all the possible relevant blocker positions
            let blocker = magic_init::gen_blockers(trunc_premask, blocker_index as u64);
            if blocker == trunc_premask {
                break;
            }
            unsafe {
                ROOK_BLOCKERS[sq][blocker_index] = blocker;
            };

            // Calculate all the possible moves
            let mut pos_mv = Vec::new();
            'offset: for offset in constants::ROOK_OFFSETS {
                let mut found_blocker = false;
                for i in 1..8 {
                    let new_pos: i8 = sq as i8 + offset * i as i8;
                    if new_pos >= 0
                        && new_pos < 64
                        && util::no_wrap(
                            (sq as i8 + offset * (i as i8 - 1)) as u8,
                            new_pos as u8,
                        )
                        && !found_blocker
                    {
                        // There is a blocker there:
                        if (blocker >> new_pos) & 1 == 1 {
                            found_blocker = true;
                        }
                        pos_mv.push(new_pos as u8);
                    } else {
                        continue 'offset;
                    }
                }
            }
            let mut movemask = board::Board::new(0);
            movemask.set_all(pos_mv);
            unsafe {
                ROOK_MOVE_WITH_BLOCKERS[sq][blocker_index] = movemask.val();
            }
            blocker_index += 1;
            if last_iteration {
                break;
            }
        }
    }
}
