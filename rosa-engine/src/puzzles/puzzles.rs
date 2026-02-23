use rosa_lib::{mv, pos::Pos};

use crate::{
    eval, fen, make, search,
    thread_search::{SearchStats, Stop},
};
use std::{
    fs,
    io::{BufRead, BufReader},
    thread,
};

use crossbeam::channel;

const PUZZLE_PATH: &str = "";

// Puzzles from: https://database.lichess.org/#puzzles
// Format: PuzzleId,FEN,Moves,Rating,RatingDeviation,Popularity,NbPlays,Themes,GameUrl,OpeningTags
// E.g.: 00008,r6k/pp2r2p/4Rp1Q/3p4/8/1N1P2R1/PqP2bPP/7K b - - 0 24,f2g3 e6e7 b2b1 b3c1 b1c1 h6c1,2037,77,95,9125,crushing hangingPiece long middlegame,https://lichess.org/787zsVup/black#48,

const THREAD_COUNT: usize = 4;

pub fn puzzle() {
    let file = fs::File::open(PUZZLE_PATH).unwrap();
    let reader = BufReader::new(file);

    let (puzzle_sender, recv) = channel::bounded(THREAD_COUNT * 3);
    for _t in 0..THREAD_COUNT {
        let recv = recv.clone();
        thread::spawn(|| puzzle_thread(recv));
    }
    drop(recv);

    for line in reader.lines() {
        let line = line.unwrap();
        let split: Vec<&str> = line.split(',').collect();

        let fen = split[1];
        let moves: Vec<&str> = split[2].split_ascii_whitespace().collect();
        let rating = split[3];

        let mut pos = fen::fen(fen.split_ascii_whitespace().collect(), Vec::new());

        let mut mvs = Vec::new();
        for m in moves {
            mvs.push(mv::Mv::new_from_str(m, &pos));
        }
        mvs.reverse();
        let mut first_mv = mvs.pop().unwrap();
        let (_legal, guard) = make::make(&mut pos, &mut first_mv, false);
        unsafe {
            guard.verified_drop();
        }

        puzzle_sender.send((pos, mvs)).unwrap();
    }
}

/// How many depths in a row does the solve get the correct solution
const MIN_CORRECT_MVS: usize = 3;
fn puzzle_thread(recv: channel::Receiver<(Pos, Vec<mv::Mv>)>) {
    for (pos, mvs) in recv.recv() {
        solve_puzzle(pos, mvs);
    }
}

fn solve_puzzle(mut pos: Pos, mut mvs: Vec<mv::Mv>) {
    let stop = Stop::new();
    let mut depth = 1;
    let mut correct_mvs = 0;
    loop {
        match search::negascout(
            &mut pos,
            depth,
            eval::SAFE_MIN_SCORE,
            eval::SAFE_MAX_SCORE,
            &mut SearchStats::new(depth),
            &stop,
        ) {
            search::SearchRes::TimeOut => panic!("Timeout"),
            search::SearchRes::Leaf(..) => continue,
            search::SearchRes::NoPonderNode(mv, ..) | search::SearchRes::Node(mv, ..) => {
                if mv == *mvs.last().unwrap() {
                    correct_mvs += 1;
                }
            }
        }

        if correct_mvs >= MIN_CORRECT_MVS {
            let mut correct = mvs.pop().unwrap();
            if mvs.len() == 0 {
                return;
            }
            let (_legal, guard) = make::make(&mut pos, &mut correct, false);
            unsafe {
                guard.verified_drop();
            }
        }
        depth += 1;
    }
}
