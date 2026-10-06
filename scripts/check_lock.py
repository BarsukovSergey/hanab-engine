"""Run a command while holding a cross-platform nonblocking file lock."""

from __future__ import annotations

import argparse
from pathlib import Path
import os
import subprocess
import sys
from typing import BinaryIO


def acquire_lock(path: Path) -> BinaryIO | None:
    path.parent.mkdir(parents=True, exist_ok=True)
    handle = path.open("a+b")
    if os.name == "nt":
        import msvcrt

        handle.seek(0, os.SEEK_END)
        if handle.tell() == 0:
            handle.write(b"\0")
            handle.flush()
        handle.seek(0)
        try:
            msvcrt.locking(handle.fileno(), msvcrt.LK_NBLCK, 1)
        except OSError:
            handle.close()
            return None
    else:
        import fcntl

        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError:
            handle.close()
            return None
    return handle


def release_lock(handle: BinaryIO) -> None:
    if os.name == "nt":
        import msvcrt

        handle.seek(0)
        msvcrt.locking(handle.fileno(), msvcrt.LK_UNLCK, 1)
    else:
        import fcntl

        fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
    handle.close()


def run_locked(path: Path, command: list[str]) -> int:
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        raise ValueError("a command is required after --")

    handle = acquire_lock(path)
    if handle is None:
        print(f"Another validation run is already active for lock: {path}", file=sys.stderr)
        return 1

    try:
        return subprocess.run(command, check=False).returncode
    finally:
        release_lock(handle)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("lock", type=Path)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    return run_locked(args.lock, args.command)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f"check-lock: {error}", file=sys.stderr)
        raise SystemExit(1)
