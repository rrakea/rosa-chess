use rosa::*;

/*
    To test the transposition table we effectivly run our move generation tests
    again (they have to all pass)
    We missuse the i32 score field in the table entry to save our count of nodes
    below. When they all match up we know key generation and saving etc. all works
*/


fn tt_counting_search(tt: table::TT, p: &pos::Pos, depth: u8) -> i32 {
    if depth == 0 {
        return 1;
    }

    let 
    
}
