#!/bin/bash
path="/home/rrakea/code/rosa"


fastchess -engine cmd="${path}/testing/baseline" name=Baseline\
    -engine cmd="${path}/target/release/rosa-engine" name=Test\
    -openings file=8moves_v3.pgn format=pgn order=random\
    -each tc=10+0.1\
    -rounds 50\
    -concurrency 4\
    -log file=${path}/testing/log.txt level=warn engine=true\

echo -e "\n\n\n\n\n" >> ${path}/testing/log.txt
