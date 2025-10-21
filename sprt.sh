#!/bin/bash

git checkout dev
cargo build --release
cp ./target/release/rosa-engine ./sprt_builds/main
git checkout main
cargo build --release
cp ./target/release/rosa-engine ./sprt_builds/dev

# Opening books from: https://github.com/official-stockfish/books
 
fastchess \
-engine cmd=./sprt_builds/main name=main restart=on \
-engine cmd=./sprt_builds/dev name=dev restart=on \
-each tc=300+5 \
-openings file=./sprt_builds/8moves_v3.pgn format=pgn order=random \
-sprt elo0=0 elo1=2 alpha=0.05 beta=0.05 \
-rounds 100 \
-concurrency 3 \
-log file=./games/log realtime=true
