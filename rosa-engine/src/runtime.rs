//! # UCI Handling
//! Spawns a separate thread to handle both stdin and timeouts

use crate::config;
use crate::eval;
use crate::eval::eval;
use crate::fen;
use crate::make;
use crate::mv;
use crate::thread_search;

use crossbeam::channel;
use crossbeam::select;

use rosa_lib::mv::Mv;
use rosa_lib::piece::Clr;
use rosa_lib::pos;
use rosa_lib::tt;

use std::os::linux::raw::stat;
use std::sync::Once;
use std::thread;
use std::time;
use std::time::Duration;

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        rosa_lib::lib_init();
        tt::init_zobrist_keys();
        mv::magic_init::init_magics();
        crate::search::TT.resize(config::TT_SIZE);
        eval::init_eval();
    });
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Start,
    TimedSearch(time::Instant, time::Duration),
    UntimedSearch,
    Pondering,
    Pause(Mv),
}

impl State {
    fn is_searching(&self) -> bool {
        matches!(self, State::TimedSearch(_, _) | State::UntimedSearch | State::Pondering)
    }

pub fn start() {
    let (tx, rx) = channel::unbounded::<String>();
    thread::spawn(move || {
        loop {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).unwrap();
            tx.send(buf).unwrap();
        }
    });

    let mut state = State::Start;
    let mut pos: pos::Pos = fen::starting_pos(Vec::new());

    loop {
        let timeout = if let State::TimedSearch(start_time, dur) = state {
            let elapsed = start_time.elapsed();
            if elapsed >= dur {
                Duration::from_millis(0)
            } else {
                dur - elapsed
            }
        } else {
            Duration::from_millis(u64::MAX)
        };

        let cmd: String;
        select! {
            recv(rx) -> c => {
                cmd = c.unwrap();
            }
            default(timeout) => {
                thread_search::stop_search();
                state = State::Pause;
                continue;
            }
        }

        let cmd_res = handle_cmd(cmd, &mut pos);
        match cmd_res {
            CmdRes::TimedSearch(duration) => {
                // If pause or start do nothing
                if state.is_searching() {
                    thread_search::stop_search();
                }
                state = State::TimedSearch(time::Instant::now(), duration);
                thread_search::threaded_search(&pos);
            }
            CmdRes::UntimedSearch => {
                state = State::UntimedSearch;
                thread_search::threaded_search(&pos);
            }
            CmdRes::Stop => {
                thread_search::stop_search();
                state = State::Pause();
            }
            CmdRes::Ponder => {
                match ponder {
                    Some(mut mv) => {
                        let mut p = pos.clone();
                        make::make(&mut p, &mut mv, false);
                        thread_search::threaded_search(&p);
                    }
                    None => panic!("No ponder move set"),
                }
            }
            CmdRes::PonderHit => {
                init();
                thread_search::stop_search();
                match ponder {
                    Some(mut mv) => {
                        make::make(&mut pos, &mut mv, false);
                        stop_time = Some((time::Instant::now(), time::Duration::from_millis(0)));
                        thread_search::threaded_search(&pos);
                    }
                    None => panic!("No ponder move set"),
                }
            }
            CmdRes::Nothing => (),
            CmdRes::Quit => break,
        }
    }
}

enum CmdRes {
    UntimedSearch,
    TimedSearch(time::Duration),
    Nothing,
    Stop,
    Ponder,
    Quit,
    PonderHit,
}

fn handle_cmd(cmd: String, pos: &mut pos::Pos) -> CmdRes {
    let cmd_parts: Vec<&str> = cmd.split_ascii_whitespace().collect();
    if cmd_parts.is_empty() {
        return CmdRes::Nothing;
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
                return CmdRes::Nothing;
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
                "startpos" => *pos = fen::starting_pos(moves),
                "fen" => *pos = fen::fen(fen, moves),
                _ => return CmdRes::Nothing,
            }
        }

        "quit" => {
            return CmdRes::Quit;
        }

        "stop" => {
            return CmdRes::Stop;
        }

        "go" => {
            if cmd_parts.len() == 1 {
                return CmdRes::UntimedSearch;
            } else {
                match cmd_parts[1] {
                    "ponder" => {
                        return CmdRes::Ponder;
                    }

                    _ => {
                        let search_duration = process_go(cmd_parts, pos.clr());
                        return CmdRes::TimedSearch(search_duration);
                    }
                }
            }
        }

        "moves" => {
            init();
            println!("Warning: Does not check legality");
            if cmd_parts.len() < 2 {
                return CmdRes::Nothing;
            }
            let mv = cmd_parts[1];
            let mut mv = Mv::new_from_str(mv, &pos);
            println!("{:?}", mv);
            make::make(pos, &mut mv, false);
        }

        "print" | "p" | "d" => {
            init();
            println!("{}", &pos);
        }

        "printfull" => {
            init();
            println!("{}", &pos.full());
        }

        "magics" => {
            mv::gen_magics::gen_magics();
        }

        "eval" => {
            init();
            let eval = eval(&pos);
            println!("Eval: {eval}");
        }

        "ponderhit" => {
            return CmdRes::PonderHit;
        }

        "setoption" => {
            println!("Options currently not supported");
        }
        _ => {}
    }
    return CmdRes::Nothing;
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
    let check_next = |cmd: &[&str], index: usize| -> u64 {
        match cmd[index].parse() {
            Ok(o) => o,
            Err(e) => {
                panic!("Value after command not int, {e}, part: {}", cmd[index + 1]);
            }
        }
    };

    let mut index = 1;

    let mut wtime = 0;
    let mut btime = 0;
    let mut winc = 0;
    let mut binc = 0;
    let mut _mvs_to_go = 0;

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
            "moves_to_go" => {
                index += 1;
                _mvs_to_go = check_next(&cmd, index)
            }
            "movetime" => {
                index += 1;
                return Duration::from_millis(check_next(&cmd, index));
            }
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
