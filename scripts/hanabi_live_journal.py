from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from typing import Any

JOURNAL_SCHEMA_VERSION = 1


@dataclass(frozen=True)
class DecisionKey:
    """Stable identity for one bot decision against one public history."""

    table_id: int
    turn: int
    history_hash: str

    def to_record(self) -> dict[str, Any]:
        return {
            "tableID": self.table_id,
            "turn": self.turn,
            "historyHash": self.history_hash,
        }


def canonical_json(value: Any) -> str:
    """Serialize JSON deterministically for fingerprints and parity checks."""
    return json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    )


def json_fingerprint(value: Any) -> str:
    payload = canonical_json(value).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def strip_transport_ids(value: Any) -> Any:
    if isinstance(value, dict):
        return {
            key: strip_transport_ids(item)
            for key, item in value.items()
            if key != "tableID"
        }
    if isinstance(value, list):
        return [strip_transport_ids(item) for item in value]
    return value


def semantic_response_fingerprint(response: dict[str, Any]) -> str:
    return json_fingerprint(strip_transport_ids(response))


def history_fingerprint(snapshot: dict[str, Any]) -> str:
    """Fingerprint the replay-relevant public state, excluding live table identity."""
    replay_state = {
        key: snapshot[key]
        for key in (
            "playerNames",
            "ourPlayerIndex",
            "spectating",
            "replay",
            "options",
            "actions",
        )
        if key in snapshot
    }
    return json_fingerprint(replay_state)


def decision_key(
    table_id: int,
    turn: int,
    snapshot: dict[str, Any],
) -> DecisionKey:
    return DecisionKey(
        table_id=table_id,
        turn=turn,
        history_hash=history_fingerprint(snapshot),
    )


def game_record(
    table_id: int,
    generation: int,
    snapshot: dict[str, Any],
    config_snapshot: dict[str, Any],
) -> dict[str, Any]:
    return {
        "schemaVersion": JOURNAL_SCHEMA_VERSION,
        "tableID": table_id,
        "generation": generation,
        "playerNames": snapshot.get("playerNames", []),
        "ourPlayerIndex": snapshot.get("ourPlayerIndex"),
        "spectating": snapshot.get("spectating", False),
        "replay": snapshot.get("replay", False),
        "options": snapshot.get("options", {}),
        "configSnapshot": config_snapshot,
    }
