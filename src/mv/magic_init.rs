use crate::mv::constants::*;
use crate::util;

pub fn init_magics() {
    reserve_lookup();
    init_premasks();
    init_lookups();
}

// Reserves the correct space in the vectors
// so we dont have to copy the vector a whole bunch of times
fn reserve_lookup() {
    for sq in 0..64 {
        unsafe {
            let rook_shift = ROOK_SHIFT[sq] as usize;
            ROOK_LOOKUP[sq].reserve(2 ^ rook_shift);

            let bishop_shift = BISHOP_SHIFT[sq] as usize;
            BISHOP_LOOKUP[sq].reserve(2 ^ bishop_shift);
        }
    }
}

fn init_premasks() {
    unsafe {
        for sq in 0..64 {
            KING_MASKS[sq] = gen_move_mask(sq, &KING_OFFSETS, 1, 0, false);
            BISHOP_PREMASKS[sq] = gen_move_mask(sq, &BISHOP_OFFSETS, 8, 0, false);
            ROOK_PREMASKS[sq] = gen_move_mask(sq, &ROOK_OFFSETS, 8, 0, false);
            KNIGHT_MASKS[sq] = gen_move_mask(sq, &KNIGHT_OFFSETS, 1, 0, false);

            BISHOP_PREMASKS_TRUNC[sq] = gen_move_mask(sq, &BISHOP_OFFSETS, 8, 0, true);
            ROOK_PREMASKS_TRUNC[sq] = gen_move_mask(sq, &ROOK_OFFSETS, 8, 0, true);
        }
    }
}

fn init_lookups() {
    for sq in 0..64 {
        let rook_trunc_premask = unsafe { ROOK_PREMASKS_TRUNC[sq] };

        let mut blocker_index = 0;
        let mut last_iteration = false;
        loop {
            // Calculate all the possible relevant blocker positions
            let rook_blocker = gen_blockers(rook_trunc_premask, blocker_index);
            // If the blockers are the same as the mask we have passed in
            // we have gone through all the blockers
            if rook_blocker == rook_trunc_premask {
                last_iteration = true;
            }

            let rook_movemask = gen_move_mask(sq, &ROOK_OFFSETS, 8, rook_blocker, false);
            unsafe {
                ROOK_LOOKUP[sq][blocker_index as usize] = rook_movemask;
            }
            blocker_index += 1;
            if last_iteration {
                break;
            }
        }

        let bishop_trunc_premask = unsafe { ROOK_PREMASKS_TRUNC[sq] };

        last_iteration = false;
        blocker_index = 0;

        loop {
            let bishop_blocker = gen_blockers(bishop_trunc_premask, blocker_index);
            if bishop_blocker == bishop_trunc_premask {
                last_iteration = true
            }
            let bishop_movemask = gen_move_mask(sq, &BISHOP_OFFSETS, 8, bishop_blocker, false);
            unsafe {
                BISHOP_LOOKUP[sq][blocker_index as usize] = bishop_movemask;
            }
            blocker_index += 1;
            if last_iteration {
                break;
            }
        }
    }
}

fn gen_move_mask(sq: usize, offset: &[i8], iterator: i8, blocker_mask: u64, truncate: bool) -> u64 {
    let mut pos_moves = Vec::new();
    let mut found_blocker = false;
    'offset: for o in offset {
        for i in 1..=iterator {
            let new_pos = (sq as i8) + (o * i);
            let next_pos = (sq as i8) + (o * (i + 1));
            let last_pos = (sq as i8) + (o * (i - 1));

            let not_out_of_bounds = new_pos >= 0 && new_pos < 64;
            let no_wrap = util::util::no_wrap(last_pos as u8, new_pos as u8);

            if truncate {
                let next_not_out_of_bounds = next_pos >= 0 && next_pos < 64;
                let next_no_wrap = util::util::no_wrap(new_pos as u8, next_pos as u8);
                if next_no_wrap && next_not_out_of_bounds {
                    continue 'offset;
                }
            }

            if not_out_of_bounds && no_wrap && !found_blocker {
                if (blocker_mask >> new_pos) & 1 == 1 {
                    found_blocker = true;
                }
                pos_moves.push(new_pos as u8);
            } else {
                continue 'offset;
            }
        }
    }
    util::mask::one_at(pos_moves)
}

fn gen_blockers(mask: u64, counter: u64) -> u64 {
    let mut res = 0;
    let mut counter_position = 0;

    for b in 0..64 {
        // If the mask has a 1 at position b
        if (mask >> b) & 1 == 1 {
            // We only need to flip the bit in res if
            // the counter actually has a 1 in that position
            if (counter >> counter_position) & 1 == 1 {
                res |= 1 << b;
            }
            // We have consumed a bit from the counter
            counter_position += 1;
        }
    }
    res
}
