#!/bin/bash
path="/home/rrakea/code/rosa"

# Output: If positive: Test is better; If negative: Test is worse
fastchess\
    -engine cmd="${path}/target/release/rosa-engine" name=Test\
    -engine cmd="${path}/testing/baseline" name=Baseline\
    -openings file=8moves_v3.pgn format=pgn order=random\
    -each tc=2+0.1\
    -rounds 100\
    -concurrency 4\
    -log file=${path}/testing/log.txt level=warn\

echo -e "\n\n\n\n\n" >> ${path}/testing/log.txt
