"""Offline parity on the recorded three-player No Variant opening; no network."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import time
from typing import Any

from hanabi_live_engine import PersistentEngine, engine_environment, validate_engine_binary
from hanabi_live_trace import TraceRecorder
from hanabi_live_verify import verify_trace


def check_action(snapshot: dict[str, Any], response: dict[str, Any]) -> None:
    action = response["action"]
    assert action["tableID"] == snapshot["tableID"]
    kind, target = action["type"], action["target"]
    if kind == 0:
        assert target in range(5)  # Observer's initial hand, server card orders.
    else:
        assert kind in (2, 3)  # Discard is illegal at the opening's eight clues.
        assert target in (1, 2)
        field = "suitIndex" if kind == 2 else "rank"
        assert any(
            event["type"] == "draw" and event["playerIndex"] == target
            and event[field] == action["value"]
            for event in snapshot["actions"]
        )


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, required=True)
    args = parser.parse_args()
    engine = args.engine.resolve()
    validate_engine_binary(engine)
    snapshot = json.loads((root / "crates/hanabi-protocol/tests/fixtures/no-variant-live-opening.json").read_text())
    assert len(snapshot["playerNames"]) == 3
    own_draws = [event for event in snapshot["actions"] if event["playerIndex"] == 0]
    assert len(own_draws) == 5
    assert all(event["suitIndex"] == event["rank"] == -1 for event in own_draws)
    options = ["--convention", "h-group", "--h-group-level", "max",
               "--include-planning-details", "--move-time-ms", "120000"]
    command = [str(engine), "live-session", *options]
    recorder = TraceRecorder(root / "target/no-variant-live", {
        "engineCommand": command, "engineTimeoutSeconds": 130,
        "origin": "offline recorded opening, not a newly played live game",
    })
    started = time.monotonic()
    session = PersistentEngine(command, 130)
    try:
        payload = {"kind": "initialize", "snapshot": snapshot}
        response = session.request(payload)
        check_action(snapshot, response)
        assert not response["planning"]["budgetExhausted"], "incomplete opening analysis"
        context = recorder.begin_decision(39, 0, len(snapshot["actions"]), 1, snapshot,
                                          {"schemaVersion": 1, "hGroupLevel": "max"})
        recorder.record_request(context, 1, payload)
        recorder.record_response(context, 1, response)
        recorder.finish_decision(context, "offline-only", action=response["action"])
        turn = {"type": "turn", "num": 0, "currentPlayerIndex": 0}
        snapshot["actions"].append(turn)
        payload = {"kind": "append", "tableID": 39, "actions": [turn]}
        incremental = session.request(payload)
        check_action(snapshot, incremental)
        rebuilt = subprocess.run([str(engine), "live-action", *options],
                                 input=json.dumps(snapshot), text=True, capture_output=True,
                                 timeout=130, check=True, env=engine_environment())
        rebuilt_response = json.loads(rebuilt.stdout)
        (recorder.run_directory / "one-shot-response.json").write_text(json.dumps(rebuilt_response, indent=2) + "\n")
        context = recorder.begin_decision(39, 0, len(snapshot["actions"]), 1, snapshot,
                                          {"schemaVersion": 1, "hGroupLevel": "max"})
        recorder.record_request(context, 1, payload)
        recorder.record_response(context, 1, incremental)
        recorder.finish_decision(context, "offline-only", action=incremental["action"])
        assert not incremental["planning"]["budgetExhausted"], "incomplete session analysis"
        assert incremental == rebuilt_response, "session / one-shot parity"
    finally:
        session.close()
    report = verify_trace(recorder.run_directory)
    report.update({"origin": "offline recorded opening", "liveSignOff": "PENDING",
                   "elapsedSeconds": time.monotonic() - started, "moveTimeMs": 120000})
    (recorder.run_directory / "verification.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report))
    print(f"Trace: {recorder.run_directory}")
    assert report["checked"] == report["matched"] == 2
    assert not report["mismatches"] and not report["errors"]
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
