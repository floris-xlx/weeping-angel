#!/usr/bin/env bash
# Deterministic *software* evidence bundle for CI.
#
# This is a reproducibility fixture for catalog/pack digests and CLI output.
# It is NOT an ISO/IEC 27001 certificate, audit opinion, or compliance claim.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-1704067200}"
export TZ=UTC
export GIT_CONFIG_GLOBAL=/dev/null
export GIT_CONFIG_SYSTEM=/dev/null

OUT="${ASSURANCE_FIXTURE_DIR:-$ROOT/target/assurance-fixture}"
rm -rf "$OUT"
mkdir -p "$OUT"

BIN=(cargo run --quiet --locked --bin weeping-angel --)

disclaimer="This is a readiness assessment and is not certification."

run_validate() {
  local dest="$1"
  mkdir -p "$dest"

  "${BIN[@]}" assurance catalog validate catalog/canonical/v1 \
    >"$dest/catalog-validate.txt"
  "${BIN[@]}" assurance catalog stats catalog/canonical/v1 \
    >"$dest/catalog-stats.txt"
  "${BIN[@]}" assurance framework validate frameworks/iso-27001/2022 \
    >"$dest/framework-validate.txt"

  printf '%s\n' "$disclaimer" >"$dest/DISCLAIMER.txt"

  {
    echo "schema=weeping-angel/assurance-fixture/v1"
    echo "kind=readiness-fixture"
    echo "not_certification=true"
    echo "git_head=$(git rev-parse HEAD)"
    echo "git_tree=$(git rev-parse HEAD^{tree})"
    echo "cargo_lock=$(sha256sum Cargo.lock | awk '{print $1}')"
    echo "catalog_digest=$(awk '/^digest:/{print $2}' "$dest/catalog-stats.txt")"
    echo "catalog_controls=$(awk '/^controls:/{print $2}' "$dest/catalog-stats.txt")"
    echo "pack_path=frameworks/iso-27001/2022"
  } >"$dest/manifest.txt"

  (cd "$dest" && find . -type f | sort | while read -r f; do
    sha256sum "$f"
  done) >"$dest/SHA256SUMS"
}

run_validate "$OUT/pass-a"
run_validate "$OUT/pass-b"

if ! diff -u "$OUT/pass-a/SHA256SUMS" "$OUT/pass-b/SHA256SUMS"; then
  echo "error: assurance fixture is not deterministic" >&2
  exit 1
fi

forbidden='ISO 27001 certified|ISO 27001 compliant|certification guaranteed|audit passed'
if grep -Eir "$forbidden" "$OUT/pass-a"; then
  echo "error: fixture contains forbidden certification language" >&2
  exit 1
fi

if ! grep -Fq "not certification" "$OUT/pass-a/catalog-validate.txt" \
  && ! grep -Fq "not certification" "$OUT/pass-a/DISCLAIMER.txt"; then
  echo "error: fixture must carry the not-certification disclaimer" >&2
  exit 1
fi

cp -a "$OUT/pass-a/." "$OUT/"
rm -rf "$OUT/pass-a" "$OUT/pass-b"

echo "ok: deterministic assurance fixture at $OUT"
cat "$OUT/manifest.txt"
