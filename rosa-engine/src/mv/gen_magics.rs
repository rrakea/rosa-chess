use crate::mv::constants;
use crate::mv::magic;
use crate::mv::magic_init;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rayon::prelude::*;

use rosa_lib::piece::*;

const MAGIC_TRIES: u64 = 100_000_000;

const MAX_BLOCKER_ROOK: usize = usize::pow(2, 12);
const MAX_BLOCKER_BISHOP: usize = usize::pow(2, 13);

// The tupel saves the magic (u64) and the shift (u32)
static mut ROOK_MAGIC: [(u64, u8); 64] = [(0, 0); 64];
static mut BISHOP_MAGIC: [(u64, u8); 64] = [(0, 0); 64];

static mut ROOK_MOVE_WITH_BLOCKERS: [[u64; MAX_BLOCKER_ROOK]; 64] = [[0; MAX_BLOCKER_ROOK]; 64];
static mut BISHOP_MOVE_WITH_BLOCKERS: [[u64; MAX_BLOCKER_BISHOP]; 64] =
    [[0; MAX_BLOCKER_BISHOP]; 64];

static mut ROOK_BLOCKERS: [[u64; MAX_BLOCKER_ROOK]; 64] = [[0; MAX_BLOCKER_ROOK]; 64];
static mut BISHOP_BLOCKERS: [[u64; MAX_BLOCKER_BISHOP]; 64] = [[0; MAX_BLOCKER_BISHOP]; 64];

pub fn gen_magics() {
    magic_init::init_premasks();

    init_movemasks(Piece::Bishop);
    init_movemasks(Piece::Rook);

    magic_generator(Piece::Bishop);
    magic_generator(Piece::Rook);

    for (sq, magic) in unsafe { ROOK_MAGIC }.iter().enumerate() {
        println!("ROOK: Sq: {sq}, Magic: {:#018x},", magic.0);
    }

    for (sq, magic) in unsafe { ROOK_MAGIC }.iter().enumerate() {
        println!("ROOK: Sq: {sq}, Shift: {},", magic.1);
    }

    for (sq, magic) in unsafe { BISHOP_MAGIC }.iter().enumerate() {
        println!("BISHOP: Sq: {sq}, Magic: {:#018x},", magic.0);
    }

    for (sq, magic) in unsafe { BISHOP_MAGIC }.iter().enumerate() {
        println!("BISHOP: Sq: {sq}, Shift: {},", magic.1);
    }
}

fn magic_generator(piece: Piece) {
    (0..64).into_par_iter().for_each(|sq| {
        let mut rng = Pcg64::from_os_rng();
        let move_mask = unsafe {
            if piece == Piece::Rook {
                constants::ROOK_PREMASKS[sq]
            } else {
                constants::BISHOP_PREMASKS[sq]
            }
        };

        'shifts: for shift in (1..13).rev() {
            // Huge number :)
            'magics: for _ in 0..MAGIC_TRIES {
                // 2 ^ shift is the max amount we can reach
                // since we truncate our index to that value
                let mut move_lookup = vec![0u64; usize::pow(2, shift as u32)];

                // Using a lot of random numbers and anding them together tends to create a lower number
                // which tends to create better magics
                // (( According to the chess wiki
                let magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();

                let blockers = unsafe {
                    if piece == Piece::Rook {
                        &ROOK_BLOCKERS[sq][..]
                    } else {
                        &BISHOP_BLOCKERS[sq][..]
                    }
                };

                'blockers: for (blocker_index, blocker) in blockers.iter().enumerate() {
                    // The blocker array might not be filled up entirely
                    if *blocker == 0 && blocker_index > 0 {
                        break 'blockers;
                    }

                    let blocker_mask = move_mask & *blocker;
                    let index = magic::magic_index(magic, shift, blocker_mask);
                    let move_with_blocker = unsafe {
                        if piece == Piece::Rook {
                            ROOK_MOVE_WITH_BLOCKERS[sq][blocker_index]
                        } else {
                            BISHOP_MOVE_WITH_BLOCKERS[sq][blocker_index]
                        }
                    };

                    if move_lookup[index] == 0 {
                        // Write the move_mask into our lookup array
                        move_lookup[index] = move_with_blocker;
                    } else {
                        // There is something already at that index
                        if move_lookup[index] == move_with_blocker {
                            // Constructive hit!
                            continue 'blockers;
                        } else {
                            // The two move boards arent the same -> magic does not work
                            continue 'magics;
                        }
                    }
                }

                // We have gone through all of the blockers and all works
                // -> The magic works!
                unsafe {
                    if piece == Piece::Rook {
                        ROOK_MAGIC[sq] = (magic, shift);
                    } else {
                        BISHOP_MAGIC[sq] = (magic, shift);
                    }
                };

                // We found a magic for this shift
                continue 'shifts;
            }

            // If we havent update the magic this shift value
            // We assume there is no magic possible at this/ a smaller shift value
            if (piece == Piece::Rook && unsafe { ROOK_MAGIC[sq].1 } != shift)
                || unsafe { BISHOP_MAGIC[sq].1 } != shift
            {
                break 'shifts;
            }
        }
    });
}

fn init_movemasks(piece: Piece) {
    for sq in 0..64 {
        let trunc_premask = unsafe {
            if piece == Piece::Rook {
                constants::ROOK_PREMASKS_TRUNC[sq]
            } else {
                constants::BISHOP_PREMASKS_TRUNC[sq]
            }
        };

        let mut last_iteration = false;

        for blocker_index in 0.. {
            // Calculate all the possible relevant blocker positions
            let blocker = magic_init::gen_blockers(trunc_premask, blocker_index);

            // If the blocker is exactly the same as the
            // most possible amount of blockers we have reached the end
            if blocker == trunc_premask {
                last_iteration = true;
            }

            unsafe {
                // Safe the blockers for later so we dont have to calc this twice
                // The arrays will be partially empty, we check for that and break early
                if piece == Piece::Rook{
                    ROOK_BLOCKERS[sq][blocker_index as usize] = blocker;
                } else {
                    BISHOP_BLOCKERS[sq][blocker_index as usize] = blocker;
                }
            };

            // Calculate all the possible moves
            let directions = if piece == Piece::Rook{
                constants::ROOK_OFFSETS
            } else {
                constants::BISHOP_OFFSETS
            };

            // Gen the actual moves
            let move_mask = magic_init::gen_move_mask(sq, &directions, 8, blocker, false);

            unsafe {
                if piece == Piece::Rook{
                    ROOK_MOVE_WITH_BLOCKERS[sq][blocker_index as usize] = move_mask;
                } else {
                    BISHOP_MOVE_WITH_BLOCKERS[sq][blocker_index as usize] = move_mask;
                }
            }

            if last_iteration {
                break;
            }
        }
    }
}
