#!/bin/bash

git checkout dev
cargo build --release
--target-dir

fastchess \
-engine cmd=rosa
