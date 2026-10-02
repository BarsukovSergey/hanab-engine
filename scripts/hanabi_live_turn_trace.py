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
    interpretations = _mapping(response.get("conventionInferences"))
    connections: list[Any] = []
    active_connection = interpretations.get("connection")
    if isinstance(active_connection, dict):
        connections.append(active_connection)
    connections.extend(_list(interpretations.get("connectionPromises")))
    return {
        "schemaVersion": TURN_TRACE_SCHEMA_VERSION,
        "decisionKey": decision_key,
        "stateBefore": {
            "snapshot": snapshot_path,
            "historyHash": decision_key.get("historyHash"),
        },
        "knowledge": _mapping(response.get("logicalDeductions")),
        "interpretations": interpretations,
        "hypotheses": [],
        "connections": connections,
        "chosenAction": response.get("action"),
        "candidates": _list(planning.get("rootActions")),
        "rejectionReasons": _list(planning.get("comparisons")),
        "complexity": None,
        "risk": None,
        "provenance": {
            "source": "hanabi-live-session",
            "response": response_path,
            "responseHash": response_hash,
            "planningPhase": planning.get("phase"),
        },
    }
