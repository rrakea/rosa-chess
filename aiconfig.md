# Rosa Chess Engine - Project Configuration

## Project Overview
Rosa is a UCI-compatible chess engine written in Rust. The project follows a modular architecture with two main crates:

### Architecture
- **rosa-engine**: Main engine implementation and UCI interface
- **rosa-lib**: Core data types and utilities

## Board Representation
- **Hybrid model**: Combines square-based and piece-based representations
- **Bitboards**: Used for efficient move generation and position evaluation
- **Magic bitboards**: For sliding piece move generation

## Key Components

### rosa-engine (`/rosa-engine/src/`)
- **search.rs**: Single-threaded search implementation (multi-threading planned for later)
  - Iterative deepening with negamax framework
  - Negascout/Principal Variation Search
- **mv/mv_gen.rs**: Move generation using magic bitboards with lazy evaluation
- **make.rs**: Separate Make() and Unmake() move functions
- **runtime.rs**: UCI implementation running in separate thread
- **eval.rs**: Position evaluation (currently simple, improvements planned)
- **fen.rs**: FEN string parsing/generation
- **config.rs**: Engine configuration

### rosa-lib (`/rosa-lib/src/`)
- **mv.rs**: Move representation as u32 containing all necessary data for position restoration
- **pos.rs**: Chess position representation with dual array/bitboard approach
- **board.rs**: Bitboard operations and utilities
- **tt.rs**: Transposition table with Zobrist hashing (2GB), uses unsafe cells for efficiency
- **piece.rs**: Piece definitions and utilities
- **clr.rs**: Color definitions
- **eval_const.rs**: Evaluation constants
- **mvvlva.rs**: MVV-LVA (Most Valuable Victim - Least Valuable Attacker) ordering

## Technical Details
- **Move Generation**: Bitboard-based with magic bitboards for sliding pieces, lazy evaluation
- **Magic Bitboards**: Pre-computed for performance (generation code included to prevent desync)
- **Search**: Iterative deepening negamax with principal variation search (single-threaded, multi-threading planned)
- **Transposition Table**: 2GB using Zobrist hashing
  - Allows concurrent read/write access for performance
  - May produce "smeared" data (half old/half new) during unlucky timing - acceptable trade-off
- **Move Representation**: Moves stored as u32 containing all data needed for position restoration
- **Evaluation**: Currently simple implementation (major improvements planned)
- **Performance**: Unsafe code is acceptable for optimization
- **Threading**: UCI runs in separate thread, search is currently single-threaded
- **Testing**: Focuses on move generation correctness and transposition table functionality

## Development Guidelines
- Performance is prioritized - unsafe code allowed for speed optimization
- Modular design with clear separation between engine logic and data types
- UCI compliance for chess GUI compatibility
