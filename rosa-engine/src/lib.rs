//! # Search
//! The search function is one of the most important pieces of any chess engine.
//! As such it uses a variety of differnet optimization techinques aiming at making
//! our search as fast as possible
//! ## Effective Branching Factor
//! One metric for measuring the usefullness of certain optimizations is the effective
//! branching factor. Calculated as the nodes at the current depth / nodes of depth - 1.
//! While in practise this can be difficult to compare between different chess engines it
//! is still a useful visualization for what we are trying to optimize for.  
//! Highly optimized chess enginges have a EBF of around 2. In theory this should mean
//! that for every depth the only check 2 moves - the principle variation move (PV move) and one candidate.
//! In practise they search the PV move at full depth and other likely moves at a reducd depth.
//! ## Move Ordering
//! In order for a lot of the optimizations to function properly we have check good moves first.
//! Statically this is done via different heuristic (i.e. killer heuristic, history heuristic)
//! and MVVLVA (most valuable victim, least valuable attacker). Dynamically it is done via iterative deepening.
//! If we hade perfect move odering we could just return our first move. Since statically analysing a move is
//! a lot cheaper than searching it we try to approximate perfect ordering as much as possible
//! ## Optimizations
//! ### Alpha Beta Pruning
//! Alpha Beta pruning is one of the fundamental algorithms of chess engines.
//! It allows to reduce the search tree (effective branching factor) from the average branching
//! of a normal chess position (~ 35 - 40) to roughly the square root.  
//! It does this without cutting any nodes out of the tree that could potentially be relevant.
//! The intuitition is that if we have already found a good counter move to a proposed move (refutation)
//! we dont have to continue searching for better counter moves.
//! ### Negascout
//! Negascout in a combination of the algorithms negamax and scout
//! Negamax is a variation of the classic Minmax algorithm for opposed games
//! Negascout is also known as PVS (Principle variation search). They are functionally equivalent
//! ### Scout
//! Scout assumes that moves later in the move ordering are likely not as good and
//! and therefore searches them in a so called null window (alpha' = -alpha - 1; beta' = -alpha)  
//! As such any move better than the current posited best move will trigger an alpha cutoff
//! which is detected and researched at a normal alpha beta window
//! While researches are costly scout still significantly reduces the branching factor
//! ### Transposition Table
//! For every position we visit we save the result in the so called transposition table.
//! At the start of every position we check if we have already visited the node.
//! If we have (and the searched depth is bigger than ours ) we can just return that result and
//! dont have to check  ourselves  
//! It massivly reduces the amount of nodes that have to be searched.
//! The intuitive reason are transpositions - Position we have already visited in the same search but through
//! a different move ordering.  
//! However there are a variety of different techniques that allow us to make more use of the transposition table.
//! Firstly we dont delete the collected data between moves. Since we have already explored likely moves extensivly
//! this allows us to speed up subsequent searches. If allowed we also run a search of the likely moves during our
//! oponents thinking time (pondering).  
//! Even if our current depth is bigger than the depth of the entry in the transposition table we still gain information
//! from checking the table. The move we thought was best is saved, which massivly improves move ordering.
//! We also gain information regarding the evaluated score which can narrow our search window
//! Additionally Late Move reductions reduce the depth of calls to our search algorithm for unlikely moves, which allows
//! us to reuse previously used calculations even when the original depth was bigger.
//! ### Iterative Deepening
//! Instead of just searching a position for as long as we have time iterative deepening starts searching at depth = 1
//! and increases this by 1 every time search finishes. While this intuitivly might not make much sense
//! using alpha beta pruning and transposition tables achive in practise a massive gain in efficiency.
//! Part of this comes from better moving ordering which massivly improves the effectiveness of alpha
//! beta pruning.
//! ### Null Move Pruning
//! Null move pruning works under the assumption that doing nothing is always worse than doing something.
//! The assumption holds in practise except for very specific scenarios (zugzwang), which occur so few times,
//! that they are not worth considering checking for.  
//! Null move pruning therefor searches using a null move (= doing nothing) before even calculating possible
//! moves in a position.  
//! This allows us to warm up our transposition table and establish a lower bound for what a move in a position
//! should be able to do. This translates into increasing our beta value, which has an effect of the
//! whole subtree.
//! ### Late move reduction
//! If we have good move it stands to reason that we dont have to check later moves as thoroughly as better scored moves.
//! As described above this also allows to "underbid" the depth of previous searches and massivly gain from
//! transposition table entries.  
//! It is important to remeber than this reductions happens at every depth, as such moves that statically evaluate as bad
//! get searched quite shallowly.   
//! There are a lot of formulas ans heuristic used to decide to what exactly we can reduce our depth.
//! Rosa Chess uses a simple formula of: if depth < 6 depth - 1 else depth/3
//! This formulat is definitly open to changes with further testing

pub mod config;
pub mod debug_search;
pub mod eval;
pub mod fen;
pub mod make;
pub mod mv;
pub mod runtime;
pub mod search;
pub mod stats;
