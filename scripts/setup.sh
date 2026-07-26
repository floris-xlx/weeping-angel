#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Building weeping-angel (+ demo)"
cargo build --bins

echo "==> Running tests"
cargo test

if [[ ! -f weeping-angel.toml ]]; then
  cp weeping-angel.example.toml weeping-angel.toml
  echo "==> Created weeping-angel.toml from example"
fi

echo
echo "Setup complete."
echo "  Scanner:  cargo run --bin weeping-angel -- --help"
echo "  Lab demo: cargo run --example weeping-angel-demo --features demo"
echo "  Full lab: ./scripts/demo-scan.sh"
