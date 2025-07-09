use crate::mv;
use crate::pos;
use crate::table;
use crate::tree_search;
use std::io::{self, BufRead};

/*
    This implements the Universal Chess Interface (UCI)

*/

const NAME: &str = "rosa-chess";
const AUTHOR: &str = "rrakea";
const VERSION: &str = "0.1";

pub fn uci_start() {
    let mut pos = pos::pos::start_pos();

    let mut time = 15;
    let mut stop = false;

    let stdin = io::stdin();
    for cmd in stdin.lock().lines() {
        let cmd = cmd.unwrap();
        let cmd_parts: Vec<&str> = cmd.split_ascii_whitespace().collect();
        match cmd_parts[0] {
            "uci" => {
                println!("id name {} {}", NAME, VERSION);
                println!("id author {}", AUTHOR);

                println!("ociok");
            }
            "isready" => {
                init();
                println!("reakyok");
            }
            "setoption" => {}
            "position" => {
                pos = pos::pos::pos_from_fen();
            }
            "go" => {
                let (eval, top_move, depth, time_taken) = tree_search::search::search(&pos, time);
                println!("bestmove {}", top_move.notation());
            }
            "stop" => {
                stop = true;
            }
            "quit" => {
                return;
            }
            _ => (),
        }
    }
}

fn init() {
    table::table::init_zobrist_keys();
    mv::magic_init::init_magics();
}
