#!/bin/sh
# Build pmd-wasm (size-optimised) and copy it to viewer/pmd.wasm.
set -e
cd "$(dirname "$0")"
# Strip machine-specific paths (home dir, toolchain, checkout) from embedded panic locations.
ROOT="$(cd .. && pwd)"
RUSTUP_HOME_DIR="${RUSTUP_HOME:-$HOME/.rustup}"
CARGO_HOME_DIR="${CARGO_HOME:-$HOME/.cargo}"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$RUSTUP_HOME_DIR=/rustup --remap-path-prefix=$CARGO_HOME_DIR=/cargo --remap-path-prefix=$ROOT=/projectmd"
cargo build -p pmd-wasm --profile wasm --target wasm32-unknown-unknown
mkdir -p ../viewer
cp target/wasm32-unknown-unknown/wasm/pmd_wasm.wasm ../viewer/pmd.wasm
ls -l ../viewer/pmd.wasm
