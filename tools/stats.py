#!/usr/bin/env python3
"""Counts lines of code and lines of test per crate, and checks the test rule.

The rule: across the workspace there is at most one line of test for every
two lines of code. A test has to earn its place by protecting behaviour that
matters, so the suite cannot grow faster than what it protects.

Counted are Rust lines that are neither blank nor only a comment.

- Everything under a crate's `tests` directory is test.
- In `src`, the unit tests of a file live in one `#[cfg(test)] mod tests`
  at the end of the file, and everything from that line on is test. The
  script refuses a file that keeps tests anywhere else, because such tests
  would be counted as code: hidden from the rule and raising the allowance.
- `benches` and `examples` count on neither side. They measure and they
  show, and neither is the product or a test of it.
- Examples in doc comments are comments and count on neither side. They are
  documentation first, and they run as tests so they cannot go stale.

Usage: stats.py [--check]
"""

import pathlib
import re
import sys

# The helper beside this script is found by its place, and importing it
# leaves no cache file behind in the repository.
sys.dont_write_bytecode = True
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from workspace import ROOT, fail, members  # noqa: E402

LIMIT = 0.5
TEST_MARK = re.compile(r"#\[(test|bench)\]|#!?\[cfg\(.*\btest\b")


def code_lines(lines):
    """Yields (index, stripped line) for lines that are not blank or comment."""
    in_block_comment = False
    for index, raw in enumerate(lines):
        line = raw.strip()
        if in_block_comment:
            in_block_comment = "*/" not in line
        elif line.startswith("/*"):
            in_block_comment = "*/" not in line
        elif line and not line.startswith("//"):
            yield index, line


def count_source(path):
    """Returns (code, test) for a file in `src`, or refuses its layout."""
    name = path.relative_to(ROOT)
    lines = path.read_text(encoding="utf-8").splitlines()
    start = next((i for i, line in enumerate(lines) if line == "#[cfg(test)]"), None)
    if start is not None:
        if lines[start + 1 : start + 2] != ["mod tests {"]:
            fail(f"{name}:{start + 2}: the line after `#[cfg(test)]` has to be `mod tests {{`.")
        end = next((i for i in range(start, len(lines)) if lines[i] == "}"), len(lines))
        if any(line.strip() for line in lines[end + 1 :]):
            fail(f"{name}:{end + 2}: the test module has to be the end of the file.")
    code = test = 0
    for index, line in code_lines(lines):
        in_tests = start is not None and index >= start
        if not in_tests and TEST_MARK.search(line):
            fail(
                f"{name}:{index + 1}: a test outside the file's test module. Unit tests "
                "live in one `#[cfg(test)] mod tests` at the end of the file, where "
                "they are counted as tests."
            )
        if in_tests:
            test += 1
        else:
            code += 1
    return code, test


def count_crate(crate):
    code = test = 0
    for path in sorted(crate.rglob("*.rs")):
        place = path.relative_to(crate).parts[0]
        if place in ("benches", "examples", "target"):
            continue
        if place == "tests":
            test += sum(1 for _ in code_lines(path.read_text(encoding="utf-8").splitlines()))
        else:
            file_code, file_test = count_source(path)
            code += file_code
            test += file_test
    return code, test


def main():
    rows = [(str(crate.relative_to(ROOT)), *count_crate(crate)) for crate in members()]
    total_code = sum(row[1] for row in rows)
    total_test = sum(row[2] for row in rows)
    rows.append(("total", total_code, total_test))

    width = max(len(row[0]) for row in rows)
    print(f"{'crate':<{width}}  {'code':>8}  {'tests':>8}  ratio")
    for name, code, test in rows:
        ratio = f"{test / code:.2f}" if code else "n/a"
        print(f"{name:<{width}}  {code:>8}  {test:>8}  {ratio:>5}")

    if "--check" in sys.argv[1:] and total_test > LIMIT * total_code:
        fail(
            f"\nThe tests are {total_test} lines for {total_code} lines of code. "
            f"The limit is {LIMIT:g} lines of test per line of code: "
            "remove tests that protect nothing, or merge tests that protect the same thing."
        )


if __name__ == "__main__":
    main()
