"""Compression that other programs read, and other programs' compression read.

Packs data with the flate crate and unpacks it with zlib, then packs it with
zlib in every way zlib has and unpacks it with the flate crate. The data is
made so that repeats of every length and from every distance occur.
"""

import json
import pathlib
import random
import subprocess
import sys
import zlib

sys.dont_write_bytecode = True
sys.path.insert(0, __file__.rsplit("/", 1)[0])

from workspace import ROOT, fail  # noqa: E402



def build() -> pathlib.Path:
    """Builds the program that packs and unpacks, and says where it is."""
    build = ["cargo", "build", "--quiet", "--locked", "-p", "ow-flate", "--example", "flate"]
    if subprocess.run(build, cwd=ROOT, check=False).returncode != 0:
        fail("The flate example does not build.")
    about = ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"]
    workspace = json.loads(subprocess.run(about, cwd=ROOT, capture_output=True, check=True).stdout)
    return pathlib.Path(workspace["target_directory"]) / "debug" / "examples" / "flate"


PROGRAM = build()


def run(mode: str, data: bytes) -> bytes:
    done = subprocess.run([PROGRAM, mode], input=data, capture_output=True, check=False)
    if done.returncode != 0:
        fail(f"flate {mode} failed on {len(data)} bytes: {done.stderr.decode().strip()}")
    return done.stdout


def repeats(source: random.Random, length: int) -> bytes:
    """Bytes in which earlier parts come back: 3 to 258 bytes long, and from
    1 to 32,768 bytes back, small and large distances alike."""
    data = bytearray(source.randbytes(64))
    while len(data) < length:
        if source.random() < 0.2:
            data += source.randbytes(source.randint(1, 40))
            continue
        back = min(len(data), int(2 ** source.uniform(0, 15)))
        for _ in range(source.randint(3, 258)):
            data.append(data[-back])
    return bytes(data[:length])


def cases() -> list[tuple[str, bytes]]:
    source = random.Random(20261109)
    text = b"".join(b"%d squared is %d. " % (n, n * n) for n in range(4000))
    return [
        ("nothing", b""),
        ("one byte", b"a"),
        ("text", text),
        ("one byte, a million times", bytes(1_000_000)),
        ("noise", source.randbytes(200_000)),
        ("repeats", repeats(source, 600_000)),
        ("bytes that are mostly small", bytes(min(255, int(source.expovariate(0.1))) for _ in range(100_000))),
    ]


def packed_by_zlib(data: bytes) -> list[tuple[str, bytes]]:
    strategies = {
        "default": zlib.Z_DEFAULT_STRATEGY,
        "fixed codes": zlib.Z_FIXED,
        "codes only": zlib.Z_HUFFMAN_ONLY,
        "runs only": zlib.Z_RLE,
        "filtered": zlib.Z_FILTERED,
    }
    streams = []
    for name, strategy in strategies.items():
        for level in (0, 1, 6, 9):
            packer = zlib.compressobj(level, zlib.DEFLATED, -15, 9, strategy)
            streams.append((f"{name}, level {level}", packer.compress(data) + packer.flush()))
    return streams


def main() -> None:
    for name, data in cases():
        if zlib.decompress(run("deflate", data), -15) != data:
            fail(f"zlib unpacks {name} to something else than was packed.")
        if zlib.decompress(run("to-zlib", data)) != data:
            fail(f"zlib unpacks the zlib stream of {name} to something else.")
        if run("from-zlib", zlib.compress(data)) != data:
            fail(f"A zlib stream of {name} is unpacked to something else.")
        for how, stream in packed_by_zlib(data):
            if run("inflate", stream) != data:
                fail(f"{name}, packed by zlib ({how}), is unpacked to something else.")
    print(f"{len(cases())} kinds of data, each packed here and by zlib in 20 ways, unpack to themselves.")


if __name__ == "__main__":
    main()
