#!/usr/bin/env bash

# $1: package name from Cargo.toml

# run with e.g.:  bash build-and-bind.sh flappy-bevy

# build
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown

# bind
cargo install -q wasm-bindgen-cli
wasm-bindgen --out-dir target --target web target/wasm32-unknown-unknown/release/"$1".wasm

# ...and then run with live-server
cargo install live-server
live-server