"""What the checks in this directory need to know about the repository."""

import pathlib
import subprocess
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent


def fail(message):
    """Ends a check with a sentence instead of a traceback."""
    print(message, file=sys.stderr)
    sys.exit(1)


def read_toml(path):
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        fail(f"{path.relative_to(ROOT)} cannot be read as TOML: {error}")


def members():
    """The directories of the workspace's crates, as the root manifest names them."""
    patterns = read_toml(ROOT / "Cargo.toml").get("workspace", {}).get("members", [])
    found = set()
    for pattern in patterns:
        found.update(path for path in ROOT.glob(pattern) if (path / "Cargo.toml").is_file())
    return sorted(found)


def files():
    """Every file git tracks or would track, relative to the root.

    Outside a git checkout, such as an unpacked source archive, it is every
    file that is not in a build or git directory.
    """
    listed = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    if listed.returncode == 0:
        names = filter(None, listed.stdout.decode("utf-8").split("\0"))
        return sorted(pathlib.Path(name) for name in names if (ROOT / name).is_file())
    skipped = {".git", "target"}
    return sorted(
        path.relative_to(ROOT)
        for path in ROOT.rglob("*")
        if path.is_file() and not skipped.intersection(path.relative_to(ROOT).parts)
    )
