use crate::config;
use crate::debug_search;
use crate::eval;
use crate::eval::eval;
use crate::fen;
use crate::make;
use crate::mv;
use crate::search;

use rosa_lib::clr::Clr;
use rosa_lib::mv::Mv;
use rosa_lib::pos;
use rosa_lib::tt;

use std::sync::Once;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::time::Duration;

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        rosa_lib::lib_init();
        tt::init_zobrist_keys();
        mv::magic_init::init_magics();
        search::TT.resize(config::TT_SIZE);
        eval::init_eval();
    });
}

pub fn start() {
    let mut p = pos::Pos::default();
    let mut ponder = None;
    let mut time: Option<(time::Instant, time::Duration)> = None;

    // Spawn stdin reader
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).unwrap();
            tx.send(buf).unwrap();
        }
    });

    // Check timeout and stdin
    loop {
        if let Some((start, duration)) = time
            && start.elapsed() > duration
        {
            ponder = search::stop_search(&mut p);
            time = None
        }

        let cmd = match rx.try_recv() {
            Err(mpsc::TryRecvError::Empty) => continue,
            Err(mpsc::TryRecvError::Disconnected) => panic!("Channel DC"),
            Ok(cmd) => cmd,
        };

        let cmd_parts: Vec<&str> = cmd.split_ascii_whitespace().collect();
        if cmd_parts.is_empty() {
            continue;
        }

        match cmd_parts[0].to_lowercase().as_str() {
            "uci" => {
                println!("id name {} {}", config::NAME, config::VERSION);
                println!("id author {}", config::AUTHOR);
                print_options();
                println!("uciok");
            }

            "isready" => {
                init();
                println!("readyok");
            }

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
                ponder = search::stop_search(&mut p);
                time = None;
            }

            "go" => {
                init();
                if p.is_default() {
                    p = fen::starting_pos(Vec::new());
                }

                if cmd_parts.len() == 1 {
                    // You dont need to set the search time vals
                    search::thread_search(&p);
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
                            debug_search::division_search(&mut p, depth);
                        }
                        "ponder" => {
                            let mut clone = p.clone();
                            if let Some(mut p) = ponder {
                                make::make(&mut clone, &mut p, false);
                            }
                            search::thread_search(&p);
                        }

                        _ => {
                            let search_duration = process_go(cmd_parts, p.clr);
                            time = Some((time::Instant::now(), search_duration));
                            search::thread_search(&p);
                        }
                    }
                }
            }

            "moves" => {
                init();
                if p.is_default() {
                    p = fen::starting_pos(Vec::new())
                }
                println!("Warning: Does not check legality");
                if cmd_parts.len() < 2 {
                    println!("No move specified");
                    continue;
                }
                let mv = cmd_parts[1];
                let mut mv = Mv::new_from_str(mv, &p);
                println!("{:?}", mv);
                make::make(&mut p, &mut mv, false);
            }

            "print" | "p" | "d" => {
                init();
                println!("{}", &p);
            }

            "printfull" => {
                init();
                println!("{}", &p.full);
            }

            "attacked" => {
                init();
                println!(
                    "{}",
                    !mv::mv_gen::square_not_attacked(
                        &p,
                        cmd_parts[1].parse().unwrap(),
                        p.clr.flip()
                    )
                );
            }

            "magics" => {
                mv::gen_magics::gen_magics();
            }

            "eval" => {
                init();
                if p.is_default() {
                    p = fen::starting_pos(Vec::new());
                }
                let eval = eval(&p);
                println!("Eval: {eval}");
            }

            "ponderhit" => {
                ponder = search::stop_search(&mut p);
            }

            "setoption" => {}
            _ => {}
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

fn process_go(cmd: Vec<&str>, color: Clr) -> time::Duration {
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

    let time = if color.is_white() {
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
