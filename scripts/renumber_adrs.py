"""Assign unique ADR numbers. Keep one file per historical prefix."""

from __future__ import annotations

import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ADR = ROOT / "docs" / "adr"

KEEP = {
    "0003-canonical-assurance-catalog-v1.md",
    "0005-continuous-assurance-scheduler.md",
    "0007-supplier-risk.md",
    "0008-isms-context.md",
    "0011-repository-guard-governance.md",
}

COLLIDE = ("0003", "0005", "0007", "0008", "0011")


def main() -> None:
    files = sorted(p.name for p in ADR.glob("*.md"))
    mapping: dict[str, str] = {}
    next_n = 14
    for name in files:
        m = re.match(r"^(\d{4})-(.+)$", name)
        if not m:
            continue
        prefix, rest = m.group(1), m.group(2)
        if prefix in COLLIDE and name not in KEEP:
            mapping[name] = f"{next_n:04d}-{rest}"
            next_n += 1

    print("renames:")
    for old, new in mapping.items():
        print(f"  {old} -> {new}")
        shutil.move(ADR / old, ADR / new)

    replacements = [(old, new) for old, new in mapping.items()]
    replacements.sort(key=lambda p: len(p[0]), reverse=True)
    text_exts = {".md", ".rs", ".toml", ".txt"}
    skip_dirs = {"target", "node_modules", ".git", ".sdd"}
    for path in ROOT.rglob("*"):
        if any(part in skip_dirs for part in path.parts):
            continue
        if path.suffix.lower() not in text_exts or not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        original = text
        for old, new in replacements:
            text = text.replace(old, new)
            old_id = old[:4]
            new_id = new[:4]
            if path.parent == ADR and path.name == new:
                text = re.sub(
                    rf'(id\s*=\s*"){old_id}(")',
                    rf"\g<1>{new_id}\2",
                    text,
                    count=1,
                )
                text = text.replace(f"# ADR {old_id}", f"# ADR {new_id}", 1)
        if text != original:
            path.write_text(text, encoding="utf-8")
            print(f"updated {path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
