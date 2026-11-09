#!/usr/bin/env python3
"""Refuses a dependency that is not on the allow list.

Every crate from outside this workspace that a manifest names has to be in
tools/allowed-deps.txt, with the reason it is there. Small, well specified
pieces are written in this repository; a dependency is for something large
and subtle that a maintained library does better. The list makes adding one
a decision somebody wrote down.
"""

import pathlib
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
TABLES = ("dependencies", "dev-dependencies", "build-dependencies")


def allowed():
    names = set()
    for line in (ROOT / "tools" / "allowed-deps.txt").read_text(encoding="utf-8").splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            names.add(line.split()[0])
    return names


def outside(table):
    """Yields the names in a dependency table that do not point into the workspace."""
    for name, spec in table.items():
        local = isinstance(spec, dict) and ("path" in spec or spec.get("workspace") is True)
        if not local:
            yield spec.get("package", name) if isinstance(spec, dict) else name


def main():
    manifests = [ROOT / "Cargo.toml", *sorted((ROOT / "crates").glob("*/Cargo.toml"))]
    used = {}
    for manifest in manifests:
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        tables = [data.get(key, {}) for key in TABLES]
        tables.append(data.get("workspace", {}).get("dependencies", {}))
        for target in data.get("target", {}).values():
            tables.extend(target.get(key, {}) for key in TABLES)
        for table in tables:
            for name in outside(table):
                used.setdefault(name, manifest.relative_to(ROOT))

    missing = sorted(set(used) - allowed())
    for name in missing:
        print(f"{used[name]}: {name} is not in tools/allowed-deps.txt", file=sys.stderr)
    if missing:
        print(
            "\nAdd each one to tools/allowed-deps.txt with the reason it is needed, "
            "or write the piece in this repository.",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
