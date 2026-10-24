"""Build and run the three-game No Variant diagnostic with a hard wall limit."""

import json
from pathlib import Path
import subprocess
import time


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    output = root / "target" / "no-variant-smoke"
    output.mkdir(parents=True, exist_ok=True)
    # Compilation is outside the game budget. Run the binary directly so a
    # timeout kills the actual test process on Windows as well as Unix.
    with (output / "build.log").open("w", encoding="utf-8") as log:
        build = subprocess.run(
            ["cargo", "test", "-p", "hanabi-search", "--locked", "--test",
             "self_play", "--no-run", "--message-format=json"],
            cwd=root, stdout=subprocess.PIPE, stderr=log, text=True, check=False,
        )
    if build.returncode:
        print(f"Build failed: {output / 'build.log'}")
        return build.returncode
    executable = None
    for line in build.stdout.splitlines():
        message = json.loads(line)
        if message.get("reason") == "compiler-artifact" and message.get("executable"):
            executable = message["executable"]
    if executable is None:
        raise RuntimeError("Cargo did not report a test executable")
    # Remove only previous output owned by this diagnostic. Otherwise an old
    # completed game can be mistaken for evidence from an interrupted new run.
    for name in ("games.ndjson", "p3v0s1.json", "p4v0s1.json", "p4v0s2.json"):
        (output / name).unlink(missing_ok=True)
    started = time.monotonic()
    timed_out = False
    with (output / "run.log").open("w", encoding="utf-8") as log:
        try:
            run = subprocess.run(
                [executable, "--ignored", "--exact", "no_variant_mvp_smoke"],
                cwd=root, stdout=log, stderr=subprocess.STDOUT, timeout=180,
                check=False,
            )
            status = run.returncode
        except subprocess.TimeoutExpired:
            timed_out = True
            status = 124
    summary = {
        "exitCode": status, "hardTimeout": timed_out,
        "elapsedSeconds": time.monotonic() - started,
        "moveTimeMs": 2000, "totalTimeSeconds": 180,
        "stage": "watchdog" if timed_out else "test",
        "prefixes": [path.name for path in output.glob("p*v0s*.json")],
    }
    if timed_out:
        prefixes = list(output.glob("p*v0s*.json"))
        if prefixes:
            latest = max(prefixes, key=lambda path: path.stat().st_mtime_ns)
            summary["firstFailure"] = json.loads(latest.read_text(encoding="utf-8"))
    (output / "run.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary))
    print(f"Game reports and exact active prefixes: {output}")
    return status


if __name__ == "__main__":
    raise SystemExit(main())
