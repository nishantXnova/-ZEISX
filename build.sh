#!/bin/bash
set -e
cd calc-wasm
rustup target add wasm32-unknown-unknown || true
if ! command -v wasm-pack >/dev/null 2>&1; then
  cargo install wasm-pack
fi
wasm-pack build --target web --out-dir ../www/pkg
