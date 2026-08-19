"""Delete superseded *.baseline.rs suites and drop their Cargo.toml [[test]] rows."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def delete_baselines() -> list[Path]:
    removed: list[Path] = []
    for path in ROOT.rglob("*.baseline.rs"):
        if "target" in path.parts or "node_modules" in path.parts:
            continue
        path.unlink()
        removed.append(path)
    return removed


def strip_cargo_baseline_tests() -> None:
    cargo = ROOT / "Cargo.toml"
    text = cargo.read_text(encoding="utf-8")
    pattern = re.compile(
        r"\n\[\[test\]\]\n(?:name = [^\n]+\n)?path = \"[^\"]+\.baseline\.rs\"\n(?:required-features = [^\n]+\n)?",
    )
    new, n = pattern.subn("\n", text)
    cargo.write_text(new, encoding="utf-8")
    print(f"removed {n} Cargo.toml baseline [[test]] rows")


def main() -> None:
    removed = delete_baselines()
    print(f"deleted {len(removed)} baseline files")
    for p in removed:
        print(f"  {p.relative_to(ROOT)}")
    strip_cargo_baseline_tests()


if __name__ == "__main__":
    main()
