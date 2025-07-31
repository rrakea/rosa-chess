#[macro_use]
pub mod util;

pub mod runtime;
pub mod board;
pub mod config;
pub mod eval;
pub mod eval_const;
pub mod fen;
pub mod mv;
pub mod piece;
pub mod pos;
pub mod search;
pub mod table;


pub fn start() {
    runtime::start();
}
