"""Flip target-suite assertions so deleted baselines must stay gone."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    pat = re.compile(
        r'(?<!!)((?:toml|cargo)\.contains\(\s*(?:&format!\(\s*)?"(?:path = \\")?tests/contracts/[^"]+\.baseline\.rs")'
    )
    exists = re.compile(
        r'(?<!!)\.join\("tests/contracts/[^"]+\.baseline\.rs"\)\s*\n\s*\.is_file\(\)'
    )
    for path in list((ROOT / "tests" / "contracts").glob("*.target.rs")) + [
        ROOT / "xtask" / "tests" / "sdd_architectural_cleanup_target.rs"
    ]:
        if not path.is_file():
            continue
        original = path.read_text(encoding="utf-8")
        updated = pat.sub(lambda m: "!" + m.group(1), original)
        updated = exists.sub(lambda m: "!" + m.group(0).replace(".is_file()", ".exists()"), updated)
        if updated != original:
            path.write_text(updated, encoding="utf-8")
            print(f"patched {path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
