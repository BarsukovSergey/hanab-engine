from __future__ import annotations

import sys
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

from hanabi_live_turn_trace import normalize_turn_trace  # noqa: E402


class TurnTraceTests(unittest.TestCase):
    def test_normalizes_existing_engine_diagnostics(self) -> None:
        trace = normalize_turn_trace(
            decision_key={
                "tableID": 7,
                "turn": 4,
                "historyHash": "abc",
            },
            snapshot_path="tables/7/decision.snapshot.json",
            response_path="tables/7/decision.response.json",
            response_hash="def",
            response={
                "action": {"tableID": 7, "type": 0, "target": 3},
                "logicalDeductions": {"ownCards": []},
                "conventionInferences": {
                    "framework": "h-group",
                    "connection": {"card": 2, "kind": "Finesse"},
                    "connectionPromises": [
                        {"cards": [2, 3], "identity": {"rank": 2}},
                    ],
                },
                "planning": {
                    "phase": "symbolic",
                    "rootActions": [{"selected": True}],
                    "comparisons": [{"reason": "ConventionPriority"}],
                },
            },
        )

        self.assertEqual(trace["schemaVersion"], 1)
        self.assertEqual(trace["stateBefore"]["historyHash"], "abc")
        self.assertEqual(trace["knowledge"], {"ownCards": []})
        self.assertEqual(trace["interpretations"], {"framework": "h-group"})
        self.assertEqual(trace["candidates"], [{"selected": True}])
        self.assertEqual(len(trace["connections"]), 2)
        self.assertEqual(trace["connections"][0]["kind"], "Finesse")
        self.assertEqual(trace["chosenAction"]["target"], 3)
        self.assertEqual(
            trace["rejectionReasons"],
            [{"reason": "ConventionPriority"}],
        )
        self.assertIsNone(trace["complexity"])
        self.assertIsNone(trace["risk"])
        self.assertEqual(trace["provenance"]["responseHash"], "def")

    def test_missing_optional_diagnostics_stay_explicitly_empty(self) -> None:
        trace = normalize_turn_trace(
            decision_key={
                "tableID": 7,
                "turn": 0,
                "historyHash": "abc",
            },
            snapshot_path="snapshot.json",
            response_path="response.json",
            response_hash="def",
            response={"action": {"tableID": 7, "type": 0, "target": 0}},
        )

        self.assertEqual(trace["knowledge"], {})
        self.assertEqual(trace["interpretations"], {})
        self.assertEqual(trace["hypotheses"], [])
        self.assertEqual(trace["connections"], [])
        self.assertEqual(trace["candidates"], [])
        self.assertEqual(trace["rejectionReasons"], [])


if __name__ == "__main__":
    unittest.main()
