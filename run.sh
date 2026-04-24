#!/bin/bash
set -euo pipefail

INPUT="tests/data/2.png"
PLUGIN_DIR="target/debug"
IP="cargo run --package ip --bin ip --"

cargo clean
cargo build --workspace
cargo test --workspace

echo "--- mirror ---"
$IP -i "$INPUT" -o tests/output/test_mirror_all.png -p mirror -d tests/config/mirror_all.json -l "$PLUGIN_DIR"
$IP -i "$INPUT" -o tests/output/test_mirror_h.png   -p mirror -d tests/config/mirror_h.json   -l "$PLUGIN_DIR"
$IP -i "$INPUT" -o tests/output/test_mirror_v.png   -p mirror -d tests/config/mirror_v.json   -l "$PLUGIN_DIR"

echo "--- blur ---"
$IP -i "$INPUT" -o tests/output/test_bb_m.png -p blur -d tests/config/blur_box_mild.json   -l "$PLUGIN_DIR"
$IP -i "$INPUT" -o tests/output/test_bb_s.png -p blur -d tests/config/blur_box_strong.json -l "$PLUGIN_DIR"
$IP -i "$INPUT" -o tests/output/test_gb.png   -p blur -d tests/config/blur_gauss.json      -l "$PLUGIN_DIR"

echo "Done."
