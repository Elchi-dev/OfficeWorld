#!/usr/bin/env python3
"""Refuses a dependency that is not on the allow list.

Every crate from outside this repository that a manifest names has to be in
tools/allowed-deps.txt, with the reason it is there. Small, well specified
pieces are written in this repository; a dependency is for something large
and subtle that a maintained library does better. The list makes adding one
a decision somebody wrote down.

The check reads every Cargo.toml in the repository, wherever it is. It
refuses a crate that is not on the list, a path that leads out of the
repository, a dependency taken from git, and a `[patch]` or `[replace]`
table, since each of those is a way for outside code to arrive under an
allowed name. In Cargo.lock it refuses a package from anywhere but
crates.io.

What it does not judge is what an allowed crate depends on in turn. Those
come with the decision to allow it, and CI looks them up in the advisory
database on every push.
"""

import pathlib
import sys

# The helper beside this script is found by its place, and importing it
# leaves no cache file behind in the repository.
sys.dont_write_bytecode = True
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from workspace import ROOT, fail, files, read_toml  # noqa: E402

TABLES = ("dependencies", "dev-dependencies", "build-dependencies")
CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"


def allowed():
    names = set()
    for line in (ROOT / "tools" / "allowed-deps.txt").read_text(encoding="utf-8").splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            names.add(line.split()[0])
    return names


def judge(name, spec, manifest, permitted):
    """Returns what is wrong with one dependency, or None."""
    if not isinstance(spec, dict):
        spec = {"version": spec}
    crate = spec.get("package", name)
    if spec.get("workspace") is True:
        return None
    if "git" in spec:
        return f"{crate} comes from git. Dependencies come from crates.io or from this repository."
    if "path" in spec:
        target = (manifest.parent / spec["path"]).resolve()
        if target != ROOT and ROOT not in target.parents:
            return f"{crate} has a path that leads out of the repository: {spec['path']}"
        return None
    if crate not in permitted:
        return f"{crate} is not in tools/allowed-deps.txt"
    return None


def main():
    permitted = allowed()
    problems = []
    for name in files():
        if name.name != "Cargo.toml":
            continue
        manifest = ROOT / name
        data = read_toml(manifest)
        tables = [data.get(key, {}) for key in TABLES]
        tables.append(data.get("workspace", {}).get("dependencies", {}))
        for target in data.get("target", {}).values():
            tables.extend(target.get(key, {}) for key in TABLES)
        for table in tables:
            for dependency, spec in table.items():
                problem = judge(dependency, spec, manifest, permitted)
                if problem:
                    problems.append(f"{name}: {problem}")
        for table in ("patch", "replace"):
            if table in data:
                problems.append(f"{name}: a [{table}] table swaps a crate for other code")

    lock = ROOT / "Cargo.lock"
    if lock.is_file():
        for package in read_toml(lock).get("package", []):
            source = package.get("source")
            if source and source != CRATES_IO:
                problems.append(f"Cargo.lock: {package['name']} comes from {source}")

    if problems:
        fail(
            "\n".join(problems)
            + "\n\nAdd a crate to tools/allowed-deps.txt with the reason it is needed, "
            "or write the piece in this repository."
        )


if __name__ == "__main__":
    main()
