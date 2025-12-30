//! # UCI Handling
//! Spawns a separate thread to handle both stdin and timeouts

use crate::config;
use crate::eval;
use crate::eval::eval;
use crate::fen;
use crate::make;
use crate::mv;
use crate::thread_search;
use crate::time;

use crossbeam::channel;
use crossbeam::select;

use rosa_lib::mv::Mv;
use rosa_lib::pos::*;
use rosa_lib::tt;

use core::panic;
use std::thread;
use std::time::{Duration, Instant};

enum State {
    UnInit,
    Init(Pos),
    Search(Pos, SearchState, channel::Receiver<Mv>),
    Pause(Pos, Mv),
}

pub enum SearchState {
    Timed(Instant, Duration),
    Untimed,
    Ponder(Duration),
}

impl State {
    fn init(self) -> Self {
        match self {
            State::UnInit => {
                // These should only be called once
                rosa_lib::lib_init();
                tt::init_zobrist_keys();
                mv::magic_init::init_magics();
                crate::search::TT.resize(config::TT_SIZE);
                eval::init_eval();

                let pos = fen::starting_pos(Vec::new());
                return State::Init(pos);
            }
            _ => {
                println!("Init while already initialized");
                return self;
            }
        }
    }

    fn start_search(mut self, search_state: SearchState) -> Self {
        match self {
            State::UnInit => {
                self = self.init();
                return self.start_search(search_state);
            }
            State::Init(p) => {
                if matches!(search_state, SearchState::Ponder(_)) {
                    panic!("Pondering without having searched first")
                }
                let rec = thread_search::start_thread_search(&p);
                return State::Search(p, search_state, rec);
            }
            State::Pause(mut p, mut ponder) => {
                if matches!(search_state, SearchState::Ponder(_)) {
                    make::make(&mut p, &mut ponder, false);
                }
                let rec = thread_search::start_thread_search(&p);
                return State::Search(p, search_state, rec);
            }
            State::Search(..) => {
                self = self.pause_search();
                return self.start_search(search_state);
            }
        }
    }

    fn pause_search(self) -> Self {
        match self {
            State::Search(p, _, rec) => {
                thread_search::stop_search();
                let ponder = rec.recv().unwrap();
                return State::Pause(p, ponder);
            }
            _ => {
                println!("Pause while not searching");
                return self;
            }
        }
    }

    fn ponder_hit(self) -> Self {
        if let State::Search(p, SearchState::Ponder(dur), rec) = self {
            return State::Search(p, SearchState::Timed(Instant::now(), dur), rec);
        } else {
            panic!("Ponder hit while not pondering")
        }
    }

    fn get_timeout(&self) -> Duration {
        if let State::Search(_, SearchState::Timed(start, dur), _) = self {
            let elapsed = start.elapsed();
            if elapsed >= *dur {
                return Duration::from_millis(0);
            } else {
                return *dur - elapsed;
            }
        }
        return Duration::from_millis(u64::MAX);
    }

    fn get_pos(&self) -> Pos {
        match self {
            State::Init(p) | State::Pause(p, _) | State::Search(p, _, _) => return p.clone(),
            State::UnInit => panic!(""),
        }
    }

    fn modify_pos(&mut self, new_pos: Pos) {
        match self {
            State::Init(p) | State::Pause(p, _) | State::Search(p, _, _) => *p = new_pos,
            State::UnInit => panic!(),
        }
    }
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

    let mut state = State::UnInit;

    loop {
        let timeout = state.get_timeout();
        let cmd: String;
        select! {
            recv(rx) -> c => {
                cmd = c.unwrap();
            }
            default(timeout) => {
                state = state.pause_search();
                continue;
            }
        }

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
                state = state.init();
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
                    "startpos" => state.modify_pos(fen::starting_pos(moves)),
                    "fen" => state.modify_pos(fen::fen(fen, moves)),
                    _ => continue,
                }
            }

            "quit" => std::process::exit(0),

            "stop" => {
                state = state.pause_search();
            }

            "go" => {
                let go_res = time::parse_time_from_go(cmd_parts, state.get_pos().clr());
                state = state.start_search(go_res);
            }

            "moves" => {
                println!("Warning: Does not check legality");
                if cmd_parts.len() < 2 {
                    continue;
                }

                let mv = cmd_parts[1];
                let mut pos = state.get_pos();
                let mut mv = Mv::new_from_str(mv, &pos);
                println!("{:?}", mv);

                make::make(&mut pos, &mut mv, false);
                state.modify_pos(pos);
            }

            "print" | "p" | "d" => {
                println!("{}", state.get_pos());
            }

            "key" => {
                println!("Key: {:?}", state.get_pos().key());
            }

            "magics" => {
                mv::gen_magics::gen_magics();
            }

            "eval" => {
                let eval = eval(&state.get_pos());
                println!("Eval: {eval}");
            }

            "ponderhit" => {
                state = state.ponder_hit();
            }

            "setoption" => {
                println!("Options currently not supported");
            }
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
