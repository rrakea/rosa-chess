//! # UCI Handling
//! Spawns a separate thread to handle both stdin and timeouts

use crate::config;
use crate::eval;
use crate::eval::eval;
use crate::fen;
use crate::make;
use crate::make::MakeGuard;
use crate::mv;
use crate::search;
use crate::thread_search;
use crate::time;
use crate::time::StartSearch;

use crossbeam::channel;
use crossbeam::select;

use rosa_lib::mv::Mv;
use rosa_lib::pos::*;
use rosa_lib::tt;

use core::panic;
use std::sync::Once;
use std::thread;
use std::time::{Duration, Instant};

enum State {
    UnInit,
    Init(Pos),
    Search(
        Pos,
        SearchState,
        channel::Receiver<Option<Mv>>,
        thread_search::Stop,
    ),
    Pause(Pos, Mv),
    NoLegalMovesPause(Pos),
}

pub enum SearchState {
    Timed(Instant, Duration),
    Untimed,
    Ponder(Duration, Mv, MakeGuard),
}
static INIT: Once = Once::new();

pub fn init() {
    // These should only be called once
    INIT.call_once(|| {
        rosa_lib::lib_init();
        tt::init_zobrist_keys();
        mv::magic_init::init_magics();
        search::TT.resize(config::TT_SIZE);
        eval::init_eval();
    });
}

impl State {
    #[must_use]
    fn init(self) -> Self {
        match self {
            State::UnInit => {
                init();
                let pos = fen::starting_pos(Vec::new());
                return State::Init(pos);
            }
            _ => {
                //println!("Init while already initialized");
                return self;
            }
        }
    }

    #[must_use]
    fn start_search(mut self, state: time::StartSearch) -> Self {
        match self {
            State::UnInit => {
                self = self.init();
                return self.start_search(state);
            }
            State::Init(p) => {
                if matches!(state, StartSearch::Ponder(..)) {}
                let search_state = match state {
                    StartSearch::Ponder(_dur) => {
                        panic!("Pondering without having searched first")
                    }
                    StartSearch::Timed(dur) => SearchState::Timed(Instant::now(), dur),
                    StartSearch::Untimed => SearchState::Untimed,
                };
                let (rec, stop) = thread_search::start_thread_search(&p);
                return State::Search(p, search_state, rec, stop);
            }
            State::Pause(mut p, mut ponder) => {
                let (rec, stop) = thread_search::start_thread_search(&p);
                let search_state = match state {
                    StartSearch::Ponder(dur) => {
                        let (_legal, guard) = make::make(&mut p, &mut ponder, false);
                        SearchState::Ponder(dur, ponder, guard)
                    }
                    StartSearch::Timed(dur) => SearchState::Timed(Instant::now(), dur),
                    StartSearch::Untimed => SearchState::Untimed,
                };
                return State::Search(p, search_state, rec, stop);
            }
            State::Search(..) => {
                self = self.pause_search();
                return self.start_search(state);
            }
            State::NoLegalMovesPause(..) => {
                panic!("No legal moves in this position")
            }
        }
    }

    #[must_use]
    fn pause_search(self) -> Self {
        match self {
            State::Search(p, _, rec, mut stop) => {
                stop.stop_search();
                let ponder = rec.recv().unwrap();
                match ponder {
                    Some(pon) => {
                        return State::Pause(p, pon);
                    }
                    None => {
                        return State::NoLegalMovesPause(p);
                    }
                }
            }
            _ => {
                println!("Pause while not searching");
                return self;
            }
        }
    }

    #[must_use]
    fn ponder_hit(self) -> Self {
        if let State::Search(p, SearchState::Ponder(dur, _, guard), rec, stop) = self {
            // SAFETY: The move has been played -> we no longer need to unmake it
            unsafe {
                guard.verified_drop();
            }
            return State::Search(p, SearchState::Timed(Instant::now(), dur), rec, stop);
        } else {
            panic!("Ponder hit while not pondering")
        }
    }

    fn get_timeout(&self) -> Duration {
        if let State::Search(_, SearchState::Timed(start, dur), _, _) = self {
            let elapsed = start.elapsed();
            if elapsed >= *dur {
                return Duration::from_millis(0);
            } else {
                return *dur - elapsed;
            }
        }
        return Duration::from_millis(u64::MAX);
    }

    fn get_pos(&self) -> &Pos {
        match self {
            State::Init(p)
            | State::Pause(p, _)
            | State::NoLegalMovesPause(p)
            | State::Search(p, _, _, _) => return p,
            State::UnInit => panic!("Command used before initialized (isready)"),
        }
    }

    #[must_use]
    fn set_pos(mut self, new_pos: Pos) -> Self {
        self = match self {
            State::Init(_) => State::Init(new_pos),
            State::Pause(_, ponder) => State::Pause(new_pos, ponder),
            State::NoLegalMovesPause(_) => State::NoLegalMovesPause(new_pos),
            State::Search(_, state, rec, stop) => State::Search(new_pos, state, rec, stop),
            State::UnInit => {
                self = self.init();
                self.set_pos(new_pos)
            }
        };
        self
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
                    "startpos" => state = state.set_pos(fen::starting_pos(moves)),
                    "fen" => state = state.set_pos(fen::fen(fen, moves)),
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
                let mut pos = state.get_pos().clone();
                let mut mv = Mv::new_from_str(mv, &pos);
                println!("{:?}", mv);

                let (_, guard) = make::make(&mut pos, &mut mv, false);
                // SAFETY: The user wants to actually make these moves
                unsafe {
                    guard.verified_drop();
                }

                state = state.set_pos(pos);
            }

            // Start division search at current node
            // Only for debugging
            "div" => {
                let mut pos = state.get_pos().clone();
                let depth = cmd_parts[1].parse().unwrap();
                search::debug_division_search(&mut pos, depth);
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

            "color" => {
                let mut pos_clone = state.get_pos().clone();
                pos_clone.flip_color();
                state = state.set_pos(pos_clone);
            }

            "ponderhit" => {
                state = state.ponder_hit();
            }

            "setoption" => {
                //println!("Options currently not supported");
            }

            "ucinewgame" => {}
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
