from __future__ import annotations

import sys
import unittest
from pathlib import Path
from typing import Any

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

from hanabi_live_journal import (  # noqa: E402
    decision_key,
    history_fingerprint,
    semantic_response_fingerprint,
)


def snapshot(table_id: int = 7) -> dict[str, Any]:
    return {
        "tableID": table_id,
        "playerNames": ["Bot", "Alice"],
        "ourPlayerIndex": 0,
        "spectating": False,
        "replay": False,
        "options": {"variantName": "No Variant", "deckPlays": False},
        "actions": [
            {"type": "turn", "currentPlayerIndex": 0, "num": 3},
        ],
    }


class JournalFingerprintTests(unittest.TestCase):
    def test_history_fingerprint_ignores_live_table_identity(self) -> None:
        self.assertEqual(
            history_fingerprint(snapshot(7)),
            history_fingerprint(snapshot(99)),
        )

    def test_history_fingerprint_is_independent_of_object_key_order(self) -> None:
        first = snapshot()
        second = dict(reversed(list(first.items())))
        second["options"] = {
            "deckPlays": False,
            "variantName": "No Variant",
        }
        self.assertEqual(
            history_fingerprint(first),
            history_fingerprint(second),
        )

    def test_history_fingerprint_changes_when_public_history_changes(self) -> None:
        first = snapshot()
        second = snapshot()
        actions = list(second["actions"])
        actions.append({"type": "turn", "currentPlayerIndex": 1, "num": 4})
        second["actions"] = actions
        self.assertNotEqual(
            history_fingerprint(first),
            history_fingerprint(second),
        )

    def test_response_fingerprint_ignores_transport_table_ids(self) -> None:
        first = {
            "action": {"tableID": 7, "type": 0, "target": 3},
            "planning": {"phase": "symbolic"},
        }
        second = {
            "action": {"tableID": 99, "type": 0, "target": 3},
            "planning": {"phase": "symbolic"},
        }
        self.assertEqual(
            semantic_response_fingerprint(first),
            semantic_response_fingerprint(second),
        )

    def test_decision_key_keeps_table_and_turn_outside_history_hash(self) -> None:
        first = decision_key(7, 3, snapshot(7))
        second = decision_key(99, 3, snapshot(99))

        self.assertNotEqual(first, second)
        self.assertEqual(first.history_hash, second.history_hash)
        self.assertEqual(
            first.to_record(),
            {
                "tableID": 7,
                "turn": 3,
                "historyHash": first.history_hash,
            },
        )


if __name__ == "__main__":
    unittest.main()
