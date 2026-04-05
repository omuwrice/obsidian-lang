#!/bin/bash
cargo build --release
sudo cp target/release/obsidian /usr/local/bin/
echo "✓ Obsidian installed. Try: obsidian run examples/hello.obs"
