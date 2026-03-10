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

const PUZZLE_PATH: &str = "lichess_db_puzzle.csv";

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

    for line in reader.lines() {
        let line = line.unwrap();
        let split: Vec<&str> = line.split(',').collect();

        let fen = split[1];
        let moves: Vec<&str> = split[2].split_ascii_whitespace().collect();
        let _rating = split[3];

        let mut pos = fen::fen(fen.split_ascii_whitespace().collect(), Vec::new());
        let mut mvs = parse_moves(&pos, moves);

        // The first move is always before the actual puzzle for some reason
        let mut first_mv = mvs.pop().unwrap();
        let (_legal, guard) = make::make(&mut pos, &mut first_mv, false);
        unsafe {
            guard.verified_drop();
        }

        puzzle_sender.send((pos, mvs)).unwrap();
    }
}

fn parse_moves(p: &Pos, mv_string: Vec<&str>) -> Vec<mv::Mv> {
    let mut p = p.clone();
    let mut mvs = Vec::new();
    let mut guards = Vec::new();
    for m in mv_string {
        let mut mv = mv::Mv::new_from_str(m, &p);
        let (_legla, g) = make::make(&mut p, &mut mv, false);
        mvs.push(mv);
        guards.push(g);
    }
    for g in guards {
        unsafe {
            g.verified_drop();
        }
    }
    mvs.reverse();
    mvs
}

fn puzzle_thread(recv: channel::Receiver<(Pos, Vec<mv::Mv>)>) {
    let mut solved_count = 0;
    let mut unsolved_count = 0;

    let mut counter = 0;
    while let Ok((pos, mvs)) = recv.recv() {
        let solve = solve_puzzle(pos, mvs);
        if solve {
            solved_count += 1;
        } else {
            unsolved_count += 1;
        }
        counter += 1;
        if counter >= 10 {
            println!("Thread Partial Result:\nSolved: {solved_count}, Unsolved: {unsolved_count}");
            counter = 0;
        }
    }
    println!("Thread Result:\nSolved: {solved_count}, Unsolved: {unsolved_count}");
}

/// How many depths in a row does the solve get the correct solution
const MIN_CORRECT_MVS: usize = 3;
const MAX_DEPTH: u8 = 12;

fn solve_puzzle(mut pos: Pos, mut mvs: Vec<mv::Mv>) -> bool {
    let stop = Stop::new();
    let mut depth = 1;
    let mut correct_mvs = 0;
    while depth <= MAX_DEPTH {
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
                if mv.loose_eq(mvs.last().unwrap()) {
                    correct_mvs += 1;
                }
            }
        }

        if correct_mvs >= MIN_CORRECT_MVS {
            let mut correct = mvs.pop().unwrap();
            if mvs.len() == 0 {
                return true;
            }
            let (_legal, guard) = make::make(&mut pos, &mut correct, false);
            unsafe {
                guard.verified_drop();
            }
            depth = 1;
            continue;
        }

        depth += 1;
    }
    false
}
