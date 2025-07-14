use crate::config;
use crate::fen;
use crate::mv;
use crate::pos;
use crate::search;
use crate::table;
use std::io::{self, BufRead};

pub fn start() {
    log4rs::init_file("log4rs.yaml", Default::default());
    log::info!("Init Log File");

    let mut tt = uci_start();

    uci_runtime(&mut tt);
}

// Score, Best Move, Depth
fn start_search(
    p: &pos::Pos,
    time: u64,
    max_depth: u64,
    tt: &mut table::TT,
) -> (f32, mv::mv::Mv, u8, u64) {
    let key = table::Key::new(p);
    let (depth, time_taken) = search::search(p, time, max_depth as u8, key.clone(), tt);

    // Look up the results in the TT table
    // This will never panic since we start the search here
    let res = tt.get(&key).unwrap();
    if res.key != key {
        // This should NEVER happen if the hashing is any good
        log::error!("Well.. fuck. Overwritten the starting position TT entry");
    }

    (res.score, res.best.clone(), depth, time_taken)
}

/*
    This implements the Universal Chess Interface (UCI)

    Unimplemented Commands:
        Debug, Register, Copyprotection

    Example:
    Interface (I); Engine(E)
        I: uci -> (use uci?)
        E: id name <name>
        E: id author <author>
        E: option name <name> type <type> default <values>
        E: uciok
        I: setoption name <name> [value <value>]
        I: isready
        // Init internals
        E: readyok
        I: position [fen <fen> | startpos] [moves <moves>]
        I: go [wtime btime depth infinite]
        I: quit

*/

fn uci_start() -> table::TT {
    let stdin = io::stdin();
    let mut tt_size = config::DEFAULT_TABLE_SIZE_MB;
    let mut tt: Option<table::TT> = None;

    for cmd in stdin.lock().lines() {
        let cmd = cmd.unwrap();
        let cmd_parts: Vec<&str> = cmd.split_ascii_whitespace().collect();
        log::info!("Command Recieved: {}", cmd);
        match cmd_parts[0].to_lowercase().as_str() {
            "uci" => {
                println!("id name {} {}", config::NAME, config::VERSION);
                println!("id author {}", config::AUTHOR);
                print_options();
                println!("ociok");
            }
            "isready" => {
                tt = Some(table::TT::new(tt_size));
                table::init_zobrist_keys();
                mv::magic_init::init_magics();
                println!("reakyok");
                break;
            }
            "setoption" => {
                match process_options(cmd_parts) {
                    Some(o) => tt_size = o,
                    None => (),
                };
            }
            "quit" => {
                log::info!("Exiting Early...");
                std::process::exit(0);
            }
            _ => {
                log::warn!("UCI setup command not understood: {}", cmd)
            }
        }
    }

    match tt {
        Some(o) => return o,
        None => {
            log::error!("Programm exited before \"isready\" command was sent");
            std::process::exit(0);
        }
    }
}

static mut COLOR: i8 = 1;
static mut INC: u64 = 0;

fn uci_runtime(tt: &mut table::TT) {
    let p: Option<pos::Pos> = None;
    let stdin = io::stdin();
    for cmd in stdin.lock().lines() {
        let cmd = cmd.unwrap();
        let cmd_parts: Vec<&str> = cmd.split_ascii_whitespace().collect();
        log::info!("Command Recieved: {}", cmd);
        match cmd_parts[0].to_lowercase().as_str() {
            "go" => {
                let (max_depth, time) = process_go(cmd_parts);
                match &p {
                    None => {
                        log::error!("Go command recieved before position was set");
                        panic!("Invalid Position");
                    }
                    Some(p) => start_search(p, time, max_depth, tt),
                };
            }
            "position" => {}
            "ponderhit" => {}
            "stop" => {}

            _ => {
                log::warn!("UCI runtime command not understood: {}", cmd)
            }
        }
    }
}

fn print_options() {
    println!(
        "option name Hash type spin default {} min {} max {}",
        config::DEFAULT_TABLE_SIZE_MB,
        config::MIN_TABLE_SIZE_MB,
        config::MAX_TABLE_SIZE_MB
    );
    if config::PONDER {
        println!("option name Ponder type check default true");
    }
    if config::SHOW_CURRENT_LINE {
        println!("option name UCI_ShowCurrLine type check default false");
    }
}

fn process_options(cmd: Vec<&str>) {}

fn process_position(cmd: Vec<&str>) {}

fn process_go(cmd: Vec<&str>) -> (u64, u64) {
    let mut index = 1;

    let mut maxdepth = 0;
    let mut wtime = 0;
    let mut btime = 0;
    let mut winc = 0;
    let mut binc = 0;
    let mut movetime = 0;
    let mut infinite = false;
    let mut ponder = false;

    while index < cmd.len() {
        let cmd_part = cmd[index];

        match cmd_part.to_lowercase().as_str() {
            "wtime" => {
                index += 1;
                wtime = check_next(&cmd, index)
            }
            "btime" => {
                index += 1;
                btime = check_next(&cmd, index)
            }
            "winc" => {
                index += 1;
                winc = check_next(&cmd, index)
            }
            "binc" => {
                index += 1;
                binc = check_next(&cmd, index)
            }
            "depth" => {
                index += 1;
                maxdepth = check_next(&cmd, index)
            }
            "movetime" => {
                index += 1;
                movetime = check_next(&cmd, index)
            }
            "ponder" => ponder = true,
            "infinite" => infinite = true,
            _ => log::warn!("Go command part not understood: {}", cmd_part),
        }
        index += 1;
    }

    let time = calc_time(movetime, wtime, btime, winc, binc, infinite, ponder);

    (maxdepth, time)
}

fn check_next(cmd: &Vec<&str>, index: usize) -> u64 {
    match cmd[index + 1].parse() {
        Ok(o) => o,
        Err(e) => {
            log::error!("Value after command not int, {}", e);
            0
        }
    }
}

fn calc_time(
    movetime: u64,
    wtime: u64,
    btime: u64,
    winc: u64,
    binc: u64,
    infinte: bool,
    ponder: bool,
) -> u64 {

    if infinte || ponder {
        return 0;
    }
    if movetime != 0 {
        return movetime;
    }

    if unsafe { COLOR == 1 } {
        unsafe { INC = winc };
        (wtime / 20) + (winc / 2)
    } else {
        unsafe { INC = binc };
        (btime / 20) + (binc / 2)
    }
}
