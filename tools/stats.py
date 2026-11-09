#!/usr/bin/env python3
"""Counts lines of code and lines of test per crate, and checks the test rule.

The rule: across the workspace there is at most one line of test for every
two lines of code. A test has to earn its place by protecting behaviour that
matters, so the suite cannot grow faster than what it protects.

Counted are Rust lines that are neither blank nor only a comment. A line is
a test line when it is in a file under a `tests` or `benches` directory, or
at or below the `#[cfg(test)]` line of a source file. Every source file keeps
its unit tests in one module at the end, so that split is exact.

Examples in doc comments are comments and count on neither side. They are
documentation first, and they run as tests so they cannot go stale.

Usage: stats.py [--check]
"""

import pathlib
import sys

LIMIT = 0.5
ROOT = pathlib.Path(__file__).resolve().parent.parent


def count(path):
    """Returns (code lines, test lines) of one Rust file."""
    whole_file_is_test = any(part in ("tests", "benches") for part in path.parts)
    code = test = 0
    in_tests = whole_file_is_test
    in_block_comment = False
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if in_block_comment:
            in_block_comment = "*/" not in line
            continue
        if not line or line.startswith("//"):
            continue
        if line.startswith("/*"):
            in_block_comment = "*/" not in line
            continue
        if line.startswith("#[cfg(test)]"):
            in_tests = True
        if in_tests:
            test += 1
        else:
            code += 1
    return code, test


def main():
    crates = sorted(p.parent for p in (ROOT / "crates").glob("*/Cargo.toml"))
    rows = []
    for crate in crates:
        code = test = 0
        for path in sorted(crate.rglob("*.rs")):
            if "target" in path.relative_to(ROOT).parts:
                continue
            file_code, file_test = count(path.relative_to(ROOT))
            code += file_code
            test += file_test
        rows.append((str(crate.relative_to(ROOT)), code, test))

    total_code = sum(row[1] for row in rows)
    total_test = sum(row[2] for row in rows)
    rows.append(("total", total_code, total_test))

    width = max(len(row[0]) for row in rows)
    print(f"{'crate':<{width}}  {'code':>8}  {'tests':>8}  ratio")
    for name, code, test in rows:
        ratio = f"{test / code:.2f}" if code else "n/a"
        print(f"{name:<{width}}  {code:>8}  {test:>8}  {ratio:>5}")

    if "--check" in sys.argv[1:] and total_test > LIMIT * total_code:
        print(
            f"\nThe tests are {total_test} lines for {total_code} lines of code. "
            f"The limit is {LIMIT:g} lines of test per line of code: "
            f"remove tests that protect nothing, or merge tests that protect the same thing.",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
