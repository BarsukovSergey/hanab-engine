from __future__ import annotations

from typing import Any

TURN_TRACE_SCHEMA_VERSION = 1


def _mapping(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def _list(value: Any) -> list[Any]:
    return value if isinstance(value, list) else []


def normalize_turn_trace(
    *,
    decision_key: dict[str, Any],
    snapshot_path: str,
    response_path: str,
    response_hash: str,
    response: dict[str, Any],
) -> dict[str, Any]:
    """Normalize the existing engine diagnostics into the stable M1 trace shape."""
    planning = _mapping(response.get("planning"))
    return {
        "schemaVersion": TURN_TRACE_SCHEMA_VERSION,
        "decisionKey": decision_key,
        "stateBefore": {
            "snapshot": snapshot_path,
            "historyHash": decision_key.get("historyHash"),
        },
        "knowledge": _mapping(response.get("logicalDeductions")),
        "interpretations": _mapping(response.get("conventionInferences")),
        "hypotheses": [],
        "connections": [],
        "chosenAction": response.get("action"),
        "candidates": _list(planning.get("rootActions")),
        "rejectionReasons": [],
        "complexity": None,
        "risk": None,
        "provenance": {
            "source": "hanabi-live-session",
            "response": response_path,
            "responseHash": response_hash,
            "planningPhase": planning.get("phase"),
        },
    }
