# Rosa Chess
Chess engine written in rust

### Board Representation:
Hybrid model (square and piece based)

### Move Generation:
Bitboard based  + magic bitboards (soon)  
Lazy move generation using iterators

### Search:
Iterative Deepening in a negamax framework  
Negascout/ Principle Variation Search  
2GB Transposition Table using Zobrist hashing  


## TODO:
Attack boards  
Magic bitboards  
Piece Square Table evaluation  
Quiescence Search  
Late move reductions  
Lazy SMP parallelisation  
Aspiration windows  
Killer Move Heuristic  
UCI Integration  


## Runtime:
main() -> run();
run(): init(); loop {search(); apply(); draw();};
search() -> negascout();
negascout() -> eval(); negascout();
