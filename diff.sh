#!/bin/bash

stockfish="stockfish"
rosa="./target/release/rosa"

mkfifo sf_in sf_out rosa_in rosa_out

$stockfish < sf_in > sf_out &
sf_pid=$!

$rosa < rosa_in > rosa_out &
rosa_pid=$!

clean() {
  kill "$sf_pid" "$rosa_pid"
  rm -f sf_in sf_out rosa_in rosa_out
  exit
}

trap clean SIGINT SIGTERM EXIT

while true; do
  read -r input || break
  
  out1=$(echo "$input" > sf_in | sort)
  out2=$(echo "$input" > rosa_in | sort)

  echo "$out1" > out1.txt
  echo "$out2" > out2.txt

  comm -3 "out1.txt" "out2.txt"
done
