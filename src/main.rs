pub mod board;
pub mod config;
pub mod eval;
pub mod fen;
pub mod mask;
pub mod mv;
pub mod pos;
pub mod runtime;
pub mod search;
pub mod table;
pub mod util;

fn main() {
    runtime::start();
}
