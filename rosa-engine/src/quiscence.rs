//! # Quiscence Search
//! With chess engines searching at a very specific depth,
//! it is a victim to the *horizon effect*
//! e.g. A position where it is unavoidable to lose a queen
//! If it is possible  to stall this capture until it is out of the current depth
//! the position might seem a lot better than just giving the queen up
//! (since the queen isnt captured in the searched depth).
//! Eval() cant catch this, since it doesnt search into the future
//! This is why we dont just stop searching when depth is don,
//! but instead only evaluate on "quiet position"
//! -> Positions where you cant easily gain a piece
//! ## Stand Pat
//! We use the current static eval as a "stand pat" - a lower bound.
//! If a capture we check is worse than the stand pat value
//! it obviously isnt a good move. (Once again based on the null move hypothesis)

use rosa_lib::pos::Pos;

use crate::{eval, make, mv::mv_gen};

const BEST_POSSIBLE_CAP: i32 = 1100;
const SAFETY_MARGIN: i32 = 200;

pub fn quiscence_search(pos: &mut Pos, mut alpha: i32, beta: i32) -> i32 {
    let mut best = eval::eval(pos);

    // Even if we dont do anything we still fall out of the window
    if best >= beta {
        return best;
    }

    // Set the lowest bound
    if best > alpha {
        alpha = best;
    }

    // Even our best possible cap couldnt raise alpha
    if best + BEST_POSSIBLE_CAP < alpha {
        return alpha;
    }

    let iter = mv_gen::gen_mvs_stages(&pos, true);
    for mut mv in iter {
        if mv.is_cap() {
            let cap_val = mv.cap_victim().val() as i32;
            // This move cant raise alpha
            if best + cap_val + SAFETY_MARGIN < alpha {
                continue;
            }
        }
        let (legal, guard) = make::make(pos, &mut mv, true);
        let score;
        match legal {
            make::Legal::ILLEGAL => {
                make::unmake(pos, mv, guard);
                continue;
            }
            make::Legal::LEGAL => {
                score = -quiscence_search(pos, -beta, -alpha);
                make::unmake(pos, mv, guard);
            }
        }

        // This move beats the window
        if score >= beta {
            return score;
        }
        if score > best {
            best = score;
        }
        if score > alpha {
            alpha = score;
        }
    }

    // If there are no legal captures we return standpat
    return best;
}
