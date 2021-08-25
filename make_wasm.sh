#!/bin/bash

# exit on first sign of error:
set -e

# update (in release mode) the wasm:
cargo build --release --lib --target wasm32-unknown-unknown --features wasm

# update the wasm bindings:
wasm-bindgen target/wasm32-unknown-unknown/release/hybridcvrp.wasm \
    --no-modules \
    --no-modules-global hybridcvrp \
    --no-typescript \
    --out-dir wasm
