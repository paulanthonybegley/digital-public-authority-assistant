#!/usr/bin/env python3
"""Verify that file:line references in lesson-plans/*.md resolve in the repo.

Convention used by the lesson plans:
  - bare `foo.rs:12`          -> backend/src/services/foo.rs   (the session's subject)
  - `routes/foo.rs:12`        -> backend/src/routes/foo.rs
  - bare `Dockerfile:7`       -> backend/Dockerfile
  - anything with `/` that exists relative to the repo root is used as-is.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LP = ROOT / "lesson-plans"

SKIP = {"node_modules", "target", ".git", "dist"}

# basename -> list of repo-relative candidate paths
BASENAMES: dict[str, list[Path]] = {}
for p in ROOT.rglob("*"):
    if any(part in SKIP for part in p.parts):
        continue
    if p.is_file():
        BASENAMES.setdefault(p.name, []).append(p.relative_to(ROOT))


def resolve(raw: str) -> Path | None:
    # 1. exact path relative to repo root
    if (ROOT / raw).exists():
        return ROOT / raw
    # 2. route/module shorthand written with an internal slash
    for prefix in ("backend/src/", "backend/", ""):
        if (ROOT / prefix / raw).exists():
            return ROOT / prefix / raw
    # 3. bare basename convention
    base = Path(raw).name
    if "/" not in raw:
        if base == "Dockerfile":
            return ROOT / "backend/Dockerfile"
        svc = ROOT / "backend/src/services" / base
        if svc.exists():
            return svc
    # 4. unique suffix match
    matches = [c for c in BASENAMES.get(base, []) if str(c).endswith(raw)]
    if len(matches) == 1:
        return ROOT / matches[0]
    return None


REF = re.compile(
    r"(?<![A-Za-z0-9/_.-])"
    r"(?P<path>(?:[A-Za-z0-9_.-]+/)*"
    r"(?:[A-Za-z0-9_.-]+\.(?:rs|ts|tsx|sql|yml|yaml|conf|toml|http|md)|Dockerfile))"
    r":(?P<lines>\d+(?:[-,]\d+)*)"
)

problems: list[str] = []
checked = 0
files = sorted(LP.rglob("*.md"))
for md in files:
    rel = md.relative_to(ROOT)
    for m in REF.finditer(md.read_text()):
        checked += 1
        raw = m.group("path")
        path = resolve(raw)
        if path is None:
            problems.append(f"{rel}: unresolved path '{raw}'")
            continue
        n = len(path.read_text(errors="replace").splitlines())
        for part in m.group("lines").replace("-", ",").split(","):
            ln = int(part)
            if not 1 <= ln <= n:
                problems.append(f"{rel}: {raw}:{ln} out of range (file has {n} lines)")

# --- named test-table check (code-map "Integration tests" table) -------------
# Rows look like: | `test_name` | `:NN` |  and the bare :NN refers to tests/api.rs
tests_rs = ROOT / "backend/tests/api.rs"
if tests_rs.exists():
    tests_lines = tests_rs.read_text(errors="replace").splitlines()
    row = re.compile(r"\|\s*`(?P<name>[a-z_][a-z0-9_]+)`\s*\|\s*`:(?P<line>\d+)`\s*\|")
    for md in files:
        for r in row.finditer(md.read_text()):
            checked += 1
            name, ln = r.group("name"), int(r.group("line"))
            if not 1 <= ln <= len(tests_lines):
                problems.append(f"{md.relative_to(ROOT)}: {name} :{ln} out of range")
            elif name not in tests_lines[ln - 1]:
                problems.append(
                    f"{md.relative_to(ROOT)}: {name} :{ln} does not match "
                    f"{tests_lines[ln - 1].strip()!r}"
                )

print(f"Scanned {len(files)} markdown files, {checked} references.")
if problems:
    print(f"\n{len(problems)} PROBLEM(S):")
    for p in problems:
        print("  -", p)
    sys.exit(1)
print("All references resolve and named test-table entries match.")
