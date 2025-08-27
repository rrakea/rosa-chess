This projects contains a UCI compatibly chess engine.

rosa-engine contains the main engine such as:
  (So far) single threaded search in search.rs
  Move generation in /mv/mv_gen.rs using magic bitboards and iterators for lazy move gen
  Move ordering in /mv/mv_order.rs relying on implementation of the mv struct to represent higher value mvs as bigger u32s
  Make() and Unmake() in make.rs currently as the same function
  Uci implementation in a seperate thred in runtime.rs

rosa-lib contains the data types such as:
  Mv represents the move as a u32, which contains all the data for reverting it to a previous position and is used in move ordering
  Pos represents a chess position using both an array based representation and a bitboard based
  Board represents bitboards
  TT represents the transposition table, using zobrist hashing. It internally uses unsafe cell and allows smearing bugs for efficiency

General Guidlines:
  It is ok to use unsafe code to speed up performance

