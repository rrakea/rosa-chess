#!/bin/bash

cd /home/rrakea/code/rosa
cargo build -r
rm ./testing/baseline
cp ./target/release/rosa-engine ./testing/baseline
