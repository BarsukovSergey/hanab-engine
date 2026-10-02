from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

from hanabi_live_journal import semantic_response_fingerprint  # noqa: E402
from hanabi_live_verify import configured_engine_command, verify_trace  # noqa: E402


class FakeEngine:
    commands: list[list[str]] = []

    def __init__(self, command: list[str], _timeout: float) -> None:
        self.command = command
        self.closed = False
        self.__class__.commands.append(command)

    def request(self, payload: dict[str, Any]) -> dict[str, Any]:
        snapshot = payload["snapshot"]
        return {
            "action": {
                "tableID": snapshot["tableID"],
                "type": 0,
                "target": 3,
            },
            "planning": {"phase": "symbolic"},
        }

    def close(self) -> None:
        self.closed = True


def write_trace(root: Path, expected_response: dict[str, Any]) -> None:
    (root / "tables" / "7").mkdir(parents=True)
    (root / "run.json").write_text(
        json.dumps(
            {
                "engineCommand": [
                    "fake-engine",
                    "live-session",
                    "--convention",
                    "h-group",
                    "--h-group-level",
                    "max",
                ],
                "engineTimeoutSeconds": 5,
            }
        ),
        encoding="utf-8",
    )
    game = {
        "schemaVersion": 1,
        "tableID": 7,
        "playerNames": ["Bot", "Alice"],
        "ourPlayerIndex": 0,
        "spectating": False,
        "replay": False,
        "options": {"variantName": "No Variant"},
        "configSnapshot": {"schemaVersion": 1, "hGroupLevel": "3"},
    }
    (root / "tables" / "7" / "game.json").write_text(
        json.dumps(game),
        encoding="utf-8",
    )
    snapshot = {
        "tableID": 7,
        "playerNames": ["Bot", "Alice"],
        "ourPlayerIndex": 0,
        "spectating": False,
        "replay": False,
        "options": {"variantName": "No Variant"},
        "actions": [{"type": "turn", "num": 4, "currentPlayerIndex": 0}],
    }
    snapshot_path = root / "tables" / "7" / "decision.snapshot.json"
    snapshot_path.write_text(json.dumps(snapshot), encoding="utf-8")
    record = {
        "schemaVersion": 1,
        "decisionId": 1,
        "decisionKey": {
            "tableID": 7,
            "turn": 4,
            "historyHash": "test",
        },
        "snapshot": "tables/7/decision.snapshot.json",
        "responseHash": semantic_response_fingerprint(expected_response),
        "status": "sent",
    }
    (root / "tables" / "7" / "decisions.jsonl").write_text(
        json.dumps(record) + "\n",
        encoding="utf-8",
    )


class ParityTests(unittest.TestCase):
    def setUp(self) -> None:
        FakeEngine.commands = []

    def test_configured_engine_command_uses_frozen_game_level(self) -> None:
        command = configured_engine_command(
            [
                "engine",
                "live-session",
                "--convention",
                "h-group",
                "--h-group-level",
                "max",
            ],
            {"configSnapshot": {"hGroupLevel": "7"}},
            None,
        )
        self.assertEqual(command[command.index("--h-group-level") + 1], "7")

    def test_verify_trace_replays_each_decision_from_fresh_snapshot(self) -> None:
        expected_response = {
            "action": {"tableID": 99, "type": 0, "target": 3},
            "planning": {"phase": "symbolic"},
        }
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            write_trace(root, expected_response)
            result = verify_trace(root, engine_factory=FakeEngine)

        self.assertEqual(result["checked"], 1)
        self.assertEqual(result["matched"], 1)
        self.assertEqual(result["mismatches"], [])
        self.assertEqual(result["errors"], [])
        self.assertEqual(len(FakeEngine.commands), 1)
        command = FakeEngine.commands[0]
        self.assertEqual(command[command.index("--h-group-level") + 1], "3")

    def test_verify_trace_reports_semantic_mismatch(self) -> None:
        expected_response = {
            "action": {"tableID": 7, "type": 0, "target": 2},
            "planning": {"phase": "symbolic"},
        }
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            write_trace(root, expected_response)
            result = verify_trace(root, engine_factory=FakeEngine)

        self.assertEqual(result["checked"], 1)
        self.assertEqual(result["matched"], 0)
        self.assertEqual(len(result["mismatches"]), 1)


if __name__ == "__main__":
    unittest.main()
