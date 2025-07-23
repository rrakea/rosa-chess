#[macro_use]
pub mod macros;

pub mod board;
pub mod config;
pub mod eval;
pub mod eval_const;
pub mod fen;
pub mod mv;
pub mod piece;
pub mod pos;
pub mod runtime;
pub mod search;
pub mod table;

fn main() {
    runtime::start();
}
