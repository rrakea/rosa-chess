pub mod board;
pub mod mvvlva;
pub mod clr;
pub mod eval_const;
pub mod mv;
pub mod piece;
pub mod pos;
pub mod tt;
pub mod util;

use std::sync::Once;

static INIT: Once = Once::new();

pub fn lib_init() {
    INIT.call_once(|| {
        mvvlva::init_mvvlva();
    });
}
