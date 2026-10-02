#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Callable

from hanabi_live_engine import EngineSession, PersistentEngine
from hanabi_live_journal import semantic_response_fingerprint

EngineFactory = Callable[[list[str], float], EngineSession]


def load_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return value


def configured_engine_command(
    base_command: list[str],
    game: dict[str, Any],
    engine_override: Path | None,
) -> list[str]:
    if not base_command:
        raise ValueError("trace run.json has an empty engineCommand")
    command = base_command.copy()
    if engine_override is not None:
        command[0] = str(engine_override.expanduser().resolve())

    try:
        convention_index = command.index("--convention")
    except ValueError:
        convention = None
    else:
        convention = (
            command[convention_index + 1]
            if convention_index + 1 < len(command)
            else None
        )
    if convention != "h-group":
        return command

    config = game.get("configSnapshot", {})
    if not isinstance(config, dict):
        raise ValueError("game configSnapshot must be a JSON object")
    level = config.get("hGroupLevel")
    if level is None:
        return command
    if not isinstance(level, str):
        raise ValueError("game hGroupLevel must be a string or null")

    try:
        level_index = command.index("--h-group-level")
    except ValueError:
        command.extend(["--h-group-level", level])
    else:
        if level_index + 1 >= len(command):
            raise ValueError("engineCommand has --h-group-level without a value")
        command[level_index + 1] = level
    return command


def verify_trace(
    run_directory: Path,
    *,
    engine_factory: EngineFactory = PersistentEngine,
    engine_override: Path | None = None,
    timeout_override: float | None = None,
) -> dict[str, Any]:
    run_directory = run_directory.expanduser().resolve()
    manifest = load_json(run_directory / "run.json")
    raw_command = manifest.get("engineCommand")
    if not isinstance(raw_command, list) or not all(
        isinstance(item, str) for item in raw_command
    ):
        raise ValueError("trace run.json engineCommand must be an array of strings")
    base_command = list(raw_command)

    raw_timeout = manifest.get("engineTimeoutSeconds", 180.0)
    timeout = timeout_override if timeout_override is not None else float(raw_timeout)
    if timeout <= 0:
        raise ValueError("engine timeout must be positive")

    checked = 0
    matched = 0
    mismatches: list[dict[str, Any]] = []
    errors: list[dict[str, Any]] = []

    tables_root = run_directory / "tables"
    if not tables_root.is_dir():
        raise ValueError("trace directory does not contain tables/")

    for table_directory in sorted(
        path for path in tables_root.iterdir() if path.is_dir()
    ):
        for game_path in sorted(table_directory.glob("game-*.json")):
            game = load_json(game_path)
            command = configured_engine_command(base_command, game, engine_override)
            generation = game.get("generation")
            if not isinstance(generation, int):
                raise ValueError(f"{game_path} generation must be an integer")
            decisions_path = (
                table_directory / f"decisions-{generation:06d}.jsonl"
            )
            if not decisions_path.is_file():
                continue
            for line_number, line in enumerate(
                decisions_path.read_text(encoding="utf-8").splitlines(),
                start=1,
            ):
            record = json.loads(line)
                if not isinstance(record, dict):
                    raise ValueError(
                        f"{decisions_path}:{line_number} must contain a JSON object"
                    )
                expected_hash = record.get("responseHash")
                snapshot_relative = record.get("snapshot")
                if not isinstance(expected_hash, str) or not isinstance(
                    snapshot_relative,
                    str,
                ):
                    continue

                checked += 1
                decision_key = record.get("decisionKey")
                try:
                    snapshot = load_json(run_directory / snapshot_relative)
                    engine = engine_factory(command, timeout)
                    try:
                        response = engine.request(
                            {"kind": "initialize", "snapshot": snapshot}
                        )
                    finally:
                        engine.close()
                    actual_hash = semantic_response_fingerprint(response)
                except (OSError, RuntimeError, TypeError, ValueError) as error:
                    errors.append(
                        {
                            "decisionKey": decision_key,
                            "error": str(error),
                        }
                    )
                    continue

                if actual_hash == expected_hash:
                    matched += 1
                    continue
                mismatches.append(
                    {
                        "decisionKey": decision_key,
                        "expectedResponseHash": expected_hash,
                        "actualResponseHash": actual_hash,
                    }
                )

    return {
        "schemaVersion": 1,
        "checked": checked,
        "matched": matched,
        "mismatches": mismatches,
        "errors": errors,
    }


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Replay Hanabi Live decision snapshots and verify deterministic parity."
    )
    parser.add_argument("trace_directory", type=Path)
    parser.add_argument(
        "--engine",
        type=Path,
        default=None,
        help="override the engine binary recorded in run.json",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=None,
        help="override the recorded per-decision engine timeout",
    )
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        result = verify_trace(
            arguments.trace_directory,
            engine_override=arguments.engine,
            timeout_override=arguments.timeout,
        )
    except (OSError, TypeError, ValueError) as error:
        print(json.dumps({"error": str(error)}, ensure_ascii=False))
        return 2

    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0 if not result["mismatches"] and not result["errors"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
