from __future__ import annotations

import json
import sys
import tempfile
import unittest
from dataclasses import FrozenInstanceError
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

from hanabi_live_config import (  # noqa: E402
    BotConfigStore,
    PersistentBotConfig,
)


class BotConfigStoreTests(unittest.TestCase):
    def test_missing_file_uses_default(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            path = Path(temporary_directory) / "bot.json"
            default = PersistentBotConfig(h_group_level="max")
            store = BotConfigStore(path, default)
            self.assertEqual(store.load(), default)

    def test_save_round_trips_and_is_versioned(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            path = Path(temporary_directory) / "bot.json"
            store = BotConfigStore(
                path,
                PersistentBotConfig(h_group_level="max"),
            )
            config = PersistentBotConfig(h_group_level="7")
            store.save(config)

            self.assertEqual(store.load(), config)
            self.assertEqual(
                json.loads(path.read_text(encoding="utf-8")),
                {"schemaVersion": 1, "hGroupLevel": "7"},
            )
            self.assertEqual(list(path.parent.glob(".*.tmp")), [])

    def test_snapshot_is_immutable_and_detached_from_future_config(self) -> None:
        config = PersistentBotConfig(h_group_level="3")
        snapshot = config.snapshot()
        updated = config.with_h_group_level("max")

        self.assertEqual(snapshot.h_group_level, "3")
        self.assertEqual(updated.h_group_level, "max")
        with self.assertRaises(FrozenInstanceError):
            setattr(snapshot, "h_group_level", "4")

    def test_rejects_unknown_schema_version(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            path = Path(temporary_directory) / "bot.json"
            path.write_text(
                '{"schemaVersion":2,"hGroupLevel":"max"}\n',
                encoding="utf-8",
            )
            store = BotConfigStore(
                path,
                PersistentBotConfig(h_group_level="max"),
            )
            with self.assertRaisesRegex(
                ValueError,
                "unsupported bot config schema version",
            ):
                store.load()

    def test_rejects_invalid_level(self) -> None:
        with self.assertRaisesRegex(ValueError, "H-Group level"):
            PersistentBotConfig(h_group_level="max").with_h_group_level("03")


if __name__ == "__main__":
    unittest.main()
