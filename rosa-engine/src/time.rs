use core::panic;
use std::time::Duration;

use rosa_lib::piece::Clr;

fn time(wtime: u64, btime: u64, winc: u64, binc: u64, clr: Clr) -> Duration {
    let time = match clr {
        Clr::White => (wtime / 20) + (winc / 2),
        Clr::Black => (btime / 20) + (binc / 2),
    };

    Duration::from_millis(time)
}

pub enum StartSearch {
    Untimed,
    Timed(std::time::Duration),
    Ponder(std::time::Duration),
}

pub fn parse_time_from_go(cmd: Vec<&str>, clr: Clr) -> StartSearch {
    let mut wtime = 0;
    let mut btime = 0;
    let mut winc = 0;
    let mut binc = 0;

    let mut ponder = false;

    if cmd.len() < 2 {
        return StartSearch::Untimed;
    }

    // Skip "go"
    let mut i = 1;

    while i < cmd.len() {
        match cmd[i] {
            "wtime" => {
                i += 1;
                wtime = cmd[i].parse().unwrap_or(0);
            }
            "btime" => {
                i += 1;
                btime = cmd[i].parse().unwrap_or(0);
            }
            "winc" => {
                i += 1;
                winc = cmd[i].parse().unwrap_or(0);
            }
            "binc" => {
                i += 1;
                binc = cmd[i].parse().unwrap_or(0);
            }
            "ponder" => {
                ponder = true;
            }
            "movetime" => {
                let movetime = cmd[i + 1].parse().unwrap_or(0);
                return StartSearch::Timed(Duration::from_millis(movetime));
            }
            "infinite" => return StartSearch::Untimed,

            // Ignore for now
            "moves_to_go" => i += 1,
            "mate" => i += 1,
            "depth" => i += 1,
            "nodes" => i += 1,

            _ => {
                panic!("Unknown go command part: {}", cmd[i])
            }
        }
        i += 1;
    }

    if !ponder {
        StartSearch::Timed(time(wtime, btime, winc, binc, clr))
    } else {
        StartSearch::Ponder(time(wtime, btime, winc, binc, clr))
    }
}
