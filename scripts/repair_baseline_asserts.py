"""Repair broken !.join syntax and forbid leftover baseline registrations."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "tests" / "contracts"


def main() -> None:
    for path in sorted(ROOT.glob("*.target.rs")):
        text = path.read_text(encoding="utf-8")
        original = text
        text = text.replace("            !.join(", "            .join(")
        text = text.replace("manifest_dir()\n            !.join(", "manifest_dir()\n            .join(")
        text = text.replace(
            'assert!(cargo.contains("path = \\"tests/contracts/controlled_documents.baseline.rs\\"");',
            'assert!(!cargo.contains("path = \\"tests/contracts/controlled_documents.baseline.rs\\"");',
        )
        if text != original:
            path.write_text(text, encoding="utf-8")
            print(f"repaired {path.name}")


if __name__ == "__main__":
    main()
