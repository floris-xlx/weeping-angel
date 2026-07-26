#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

PORT="${PORT:-8787}"
BASE="http://127.0.0.1:${PORT}/"

echo "==> Building"
cargo build --bins

echo "==> Starting demo on ${BASE}"
PORT="$PORT" cargo run --quiet --example weeping-angel-demo --features demo &
DEMO_PID=$!
cleanup() { kill "$DEMO_PID" 2>/dev/null || true; }
trap cleanup EXIT

for i in $(seq 1 30); do
  if curl -sf "$BASE" >/dev/null; then break; fi
  sleep 0.2
done

OUT="report-lab"
echo "==> Scanning lab"
cargo run --quiet --bin weeping-angel -- scan "$BASE" \
  --i-own-this \
  --allow-host 127.0.0.1 \
  --profile deep \
  --enable-active \
  --probe xss,sqli,open-redirect,path-traversal \
  --cookie "session=admin-session" \
  --compare-auth \
  --ignore-robots \
  --rps 20 \
  --depth 2 \
  --max-urls 120 \
  --fail-on high \
  -o "$OUT" \
  --format terminal,json,html,sarif

echo "Reports under ${OUT}.*"
