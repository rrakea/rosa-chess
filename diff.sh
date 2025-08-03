#!/bin/bash

stockfish="stockfish"
rosa="./target/release/rosa"

while true; do
  read -r input
  
  out1=$(echo "$input" | "$stockfish" | sort)
  out2=$(echo "$input" | "$rosa" | sort)

  echo "$out1" > out1.txt
  echo "$out2" > out2.txt

  comm -3 "out1.txt" "out2.txt"
done
