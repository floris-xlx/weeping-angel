from pathlib import Path

root = Path("tests/contracts")
for path in root.glob("*.target.rs"):
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    out = []
    changed = False
    for line in lines:
        if "_baseline" in line and "contains" in line and "!" not in line:
            newline = (
                line.replace("toml.contains", "!toml.contains")
                .replace("cargo.contains", "!cargo.contains")
                .replace("text_has(", "!text_has(")
            )
            if newline != line:
                changed = True
                line = newline
        out.append(line)
    if changed:
        path.write_text("".join(out), encoding="utf-8")
        print(path.name)
