#!/usr/bin/env python3
"""Refuses em dashes and en dashes anywhere in the repository.

A comma, a colon or a full stop says it better, and a hyphen does for a
range. The check covers every file git tracks or would track, so it holds for
code, comments, documents and configuration alike, and it covers the
messages of the commits it can see. It also refuses the HTML entities for
the two dashes, and a text file that is not UTF-8, which it could not read.
"""

import pathlib
import re
import subprocess
import sys

# The helper beside this script is found by its place, and importing it
# leaves no cache file behind in the repository.
sys.dont_write_bytecode = True
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from workspace import ROOT, fail, files  # noqa: E402

DASHES = {chr(0x2013): "en dash", chr(0x2014): "em dash"}
ENTITY = re.compile(r"&(mdash|ndash|#8211|#8212|#x2013|#x2014);", re.IGNORECASE)
# Formats that are not text. Anything else has to be UTF-8.
BINARY = {".png", ".jpg", ".jpeg", ".gif", ".webp", ".ico", ".pdf", ".zip", ".gz"}
BINARY |= {".ttf", ".otf", ".woff", ".woff2", ".docx", ".xlsx", ".pptx", ".odt", ".ods", ".odp"}


def dashes_in(text):
    """Yields (line number, what was found) for every dash in a text."""
    for number, line in enumerate(text.splitlines(), start=1):
        for dash, label in DASHES.items():
            if dash in line:
                yield number, label
        for entity in ENTITY.finditer(line):
            yield number, f"the entity {entity.group(0)}"


def main():
    found = []
    for name in files():
        if name.suffix.lower() in BINARY:
            continue
        try:
            text = (ROOT / name).read_text(encoding="utf-8")
        except UnicodeDecodeError:
            found.append(f"{name}: not UTF-8, so it cannot be checked")
            continue
        found.extend(f"{name}:{number}: {label}" for number, label in dashes_in(text))

    log = subprocess.run(
        ["git", "log", "--format=%h%x00%B%x01"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    if log.returncode == 0:
        for entry in filter(str.strip, log.stdout.decode("utf-8", "replace").split("\x01")):
            commit, _, message = entry.strip().partition("\0")
            found.extend(
                f"commit {commit}, line {number} of its message: {label}"
                for number, label in dashes_in(message)
            )

    if found:
        fail(
            "\n".join(found)
            + f"\n\n{len(found)} found. Use a comma, a colon or a full stop, "
            "and a hyphen in a range."
        )


if __name__ == "__main__":
    main()
