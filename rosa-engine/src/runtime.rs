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
use rosa_lib::pos;
use rosa_lib::tt;

use core::panic;
use std::sync::Once;
use std::thread;
use std::time::{Duration, Instant};

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

enum State {
    Start,
    Search(SearchState, channel::Receiver<Mv>),
    Pause(Mv),
}

pub enum SearchState {
    Timed(Instant, Duration),
    Untimed,
    Ponder(Duration),
}

impl State {
    fn pause_search(&mut self) {
        match self {
            State::Search(_, rec) => {
                thread_search::stop_search();
                let ponder = rec.recv().unwrap();
                *self = State::Pause(ponder);
            }
            State::Pause(_) => (),
            _ => panic!("Pause while not initialized"),
        }
    }

    fn start_search(&mut self, p: &pos::Pos, search_state: SearchState) {
        let mut ponder = None;
        match self {
            State::Pause(p) => {
                ponder = Some(p);
            }
            State::Search(_, _) => {
                self.pause_search();
                ponder = match self {
                    State::Pause(p) => Some(p),
                    _ => unreachable!(),
                }
            }
            _ => (),
        }

        let rec = match (ponder, matches!(search_state, SearchState::Ponder(_))) {
            (None, true) => panic!("Starting ponder search while not paused"),
            (Some(mv), true) => {
                let mut pclone = p.clone();
                make::make(&mut pclone, mv, false);
                thread_search::start_thread_search(&pclone)
            }
            (_, false) => thread_search::start_thread_search(p),
        };

        *self = State::Search(search_state, rec);
    }

    fn ponder_hit(&mut self) {
        match self {
            State::Search(search_state, rec) => match search_state {
                SearchState::Ponder(dur) => {
                    *self = State::Search(SearchState::Timed(Instant::now(), *dur), rec.clone());
                }
                _ => panic!("Ponder hit while not pondering"),
            },
            _ => panic!("Ponder hit while not pondering"),
        }
    }

    fn get_timeout(&self) -> Duration {
        if let State::Search(search_state, _) = self
            && let SearchState::Timed(start, dur) = search_state
        {
            let elapsed = start.elapsed();
            if elapsed >= *dur {
                return Duration::from_millis(0);
            } else {
                return *dur - elapsed;
            }
        }
        return Duration::from_millis(u64::MAX);
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

    let mut state = State::Start;
    let mut pos: pos::Pos = fen::starting_pos(Vec::new());

    loop {
        let timeout = state.get_timeout();
        let cmd: String;
        select! {
            recv(rx) -> c => {
                cmd = c.unwrap();
            }
            default(timeout) => {
                state.pause_search();
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
                    "startpos" => pos = fen::starting_pos(moves),
                    "fen" => pos = fen::fen(fen, moves),
                    _ => continue,
                }
            }

            "quit" => return,

            "stop" => {
                state.pause_search();
            }

            "go" => {
                let go_res = time::parse_time_from_go(cmd_parts, pos.clr());
                state.start_search(&pos, go_res);
            }

            "moves" => {
                init();
                println!("Warning: Does not check legality");
                if cmd_parts.len() < 2 {
                    continue;
                }
                let mv = cmd_parts[1];
                let mut mv = Mv::new_from_str(mv, &pos);
                println!("{:?}", mv);
                make::make(&mut pos, &mut mv, false);
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
                state.ponder_hit();
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
