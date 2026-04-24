#!/bin/bash

cargo clean
cargo build
cargo run --package ip --bin ip -- -i tests/data/2.png -o tests/output/test_mirror_all.png -p mirror -d tests/config/mirror_all.json -l target/debug

cargo run --package ip --bin ip -- -i tests/data/2.png -o tests/output/test_mirror_h.png -p mirror -d tests/config/mirror_h.json -l target/debug

cargo run --package ip --bin ip -- -i tests/data/2.png -o tests/output/test_mirror_v.png -p mirror -d tests/config/mirror_v.json -l target/debug

cargo run --package ip --bin ip -- -i tests/data/2.png -o tests/output/test_bb_m.png -p blur -d tests/config/blur_box_mild.json -l target/debug

cargo run --package ip --bin ip -- -i tests/data/2.png -o tests/output/test_bb_s.png -p blur -d tests/config/blur_box_strong.json -l target/debug

cargo run --package ip --bin ip -- -i tests/data/2.png -o tests/output/test_gb.png -p blur -d tests/config/blur_gauss.json -l target/debug