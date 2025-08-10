use crate::config;
use crate::debug;
use crate::fen;
use crate::mv;
use crate::search;

use rosa_lib::mv::Mv;
use rosa_lib::pos;
use rosa_lib::tt;

use std::io::{self, BufRead};
use std::sync::{Arc, RwLock};
use std::time;
use std::time::Duration;

pub fn start() {
    let stdin = io::stdin();

    tt::init_zobrist_keys();
    search::TT.resize(config::DEFAULT_TABLE_SIZE_MB * config::MB);
    mv::magic_init::init_magics();
    let mut p: pos::Pos = fen::starting_pos(Vec::new());
    let mut stop: Option<Arc<RwLock<bool>>> = None;

    for cmd in stdin.lock().lines() {
        let cmd = cmd.unwrap();
        if debug::print_uci_commands() {
            println!("UCI Command: {cmd}")
        }

        let cmd_parts: Vec<&str> = cmd.split_ascii_whitespace().collect();
        match cmd_parts[0].to_lowercase().as_str() {
            "uci" => {
                println!("id name {} {}", config::NAME, config::VERSION);
                println!("id author {}", config::AUTHOR);
                print_options();
                println!("uciok");
            }

            "isready" => println!("readyok"),

            "position" => {
                if cmd_parts.len() == 1 {
                    continue;
                }
                let split = cmd.split_once(" moves ");
                let fen: Vec<&str>;
                let mut moves = Vec::new();
                match split {
                    Some((f, m)) => {
                        fen = f.split_ascii_whitespace().collect();
                        moves = m.split_ascii_whitespace().collect();
                    }
                    None => {
                        fen = cmd_parts[2..].to_vec();
                    }
                }

                match cmd_parts[1] {
                    "startpos" => p = fen::starting_pos(moves),
                    "fen" => p = fen::fen(fen, moves),
                    _ => continue,
                }
            }

            "quit" => {
                std::process::exit(0);
            }

            "stop" => {
                if let Some(stop) = &stop {
                    let mut s = stop.write().unwrap();
                    *s = true
                }
            }

            "go" => {
                if cmd_parts.len() == 1 {
                    stop = Some(search::thread_search(&p, time::Duration::ZERO));
                } else {
                    match cmd_parts[1] {
                        "perft" => {
                            let depth = if cmd_parts.len() <= 2 {
                                6
                            } else {
                                cmd_parts[2]
                                    .parse()
                                    .expect("Depth value in perft command not num")
                            };
                            search::division_search(&p, depth);
                        }
                        _ => {
                            let time = process_go(cmd_parts, p.active);
                            stop = Some(search::thread_search(&p, time));
                        }
                    }
                }
            }

            "moves" => {
                println!("Warning: Does not check legality");
                if cmd_parts.len() < 2 {
                    println!("No move specified");
                    continue;
                }
                let mv = cmd_parts[1];
                let mv = Mv::from_str(mv, &p);
                println!("{}", mv.prittify());
                p = mv::mv_apply::apply(&p, &mv).unwrap();
            }

            "print" | "p" | "d" => {
                println!("{}", &p.prittify_sq_based());
            }

            "printfull" => {
                println!("{}", &p.prittify());
            }

            "stats" => {
                let (valid, null, size) = search::TT.usage();
                println!("Valid: {valid}, Null: {null}, Total: {size}");
            }

            "attacked" => {
                println!(
                    "{}",
                    !mv::mv_gen::square_not_attacked(&p, cmd_parts[1].parse().unwrap(), -p.active)
                );
            }

            "magics" => {
                mv::gen_magics::gen_magics();
            }

            "ponderhit" => {}
            "setoption" => {}
            _ => {
                if debug::print_uci_commands() {
                    println!("UCI setup command not understood: {cmd}");
                }
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

fn process_go(cmd: Vec<&str>, color: i8) -> time::Duration {
    let mut index = 1;

    let mut wtime = 0;
    let mut btime = 0;
    let mut winc = 0;
    let mut binc = 0;

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
            "movetime" => return Duration::from_millis(check_next(&cmd, index)),
            "ponder" | "infinite" => return Duration::from_millis(0),
            _ => println!("Go command part not understood: {cmd_part}"),
        }
        index += 1;
    }
    if wtime + btime + winc + binc == 0 {
        return Duration::from_millis(0);
    }

    let time = if color == 1 {
        (wtime / 20) + (winc / 2)
    } else {
        (btime / 20) + (binc / 2)
    };

    Duration::from_millis(time)
}

fn check_next(cmd: &[&str], index: usize) -> u64 {
    match cmd[index].parse() {
        Ok(o) => o,
        Err(e) => {
            println!("Value after command not int, {e}, part: {}", cmd[index + 1]);
            0
        }
    }
}
