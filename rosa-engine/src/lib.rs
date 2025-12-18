//! Documentation for the rosa chess engine
//! The code is split into 2 modules:  
//! __rosa-lib__ for struct and static code  
//! __rosa-engine__ for runtime code  

pub mod config;
pub mod eval;
pub mod fen;
pub mod make;
pub mod mv;
pub mod runtime;
pub mod search;
pub mod thread_search;