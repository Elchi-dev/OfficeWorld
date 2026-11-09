#!/usr/bin/env python3
"""Refuses em dashes and en dashes anywhere in the repository.

A comma, a colon or a full stop says it better, and a hyphen does for a
range. The check covers every file git tracks or would track, so it holds for
code, comments, documents and configuration alike.
"""

import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
DASHES = {chr(0x2013): "en dash", chr(0x2014): "em dash"}


def main():
    listed = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout
    found = 0
    for name in filter(None, listed.decode("utf-8").split("\0")):
        path = ROOT / name
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        for number, line in enumerate(text.splitlines(), start=1):
            for dash, label in DASHES.items():
                if dash in line:
                    print(f"{name}:{number}: {label}", file=sys.stderr)
                    found += 1
    if found:
        print(
            f"\n{found} dash(es) found. Use a comma, a colon or a full stop, "
            "and a hyphen in a range.",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
