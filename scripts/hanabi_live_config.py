from __future__ import annotations

import json
import os
import threading
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any

CONFIG_SCHEMA_VERSION = 1


@dataclass(frozen=True)
class GameConfigSnapshot:
    """Immutable gameplay configuration frozen when a live game starts."""

    schema_version: int
    h_group_level: str | None

    def to_record(self) -> dict[str, Any]:
        return {
            "schemaVersion": self.schema_version,
            "hGroupLevel": self.h_group_level,
        }


@dataclass(frozen=True)
class PersistentBotConfig:
    """Sticky bot configuration inherited by future games."""

    schema_version: int = CONFIG_SCHEMA_VERSION
    h_group_level: str | None = None

    def snapshot(self) -> GameConfigSnapshot:
        return GameConfigSnapshot(
            schema_version=self.schema_version,
            h_group_level=self.h_group_level,
        )

    def with_h_group_level(self, level: str | None) -> PersistentBotConfig:
        BotConfigStore.validate_level(level)
        return replace(self, h_group_level=level)


class BotConfigStore:
    """Versioned, atomically persisted configuration for the live bot."""

    def __init__(self, path: Path, default: PersistentBotConfig) -> None:
        self.path = path
        self.default = default
        self.lock = threading.RLock()

    def load(self) -> PersistentBotConfig:
        with self.lock:
            try:
                raw = json.loads(self.path.read_text(encoding="utf-8"))
            except FileNotFoundError:
                return self.default
            except (OSError, json.JSONDecodeError) as error:
                raise ValueError(f"could not load bot config: {error}") from error

            if not isinstance(raw, dict):
                raise ValueError("bot config must be a JSON object")
            version = raw.get("schemaVersion")
            if version != CONFIG_SCHEMA_VERSION:
                raise ValueError(
                    f"unsupported bot config schema version: {version!r}"
                )
            level = raw.get("hGroupLevel")
            self.validate_level(level)
            return PersistentBotConfig(
                schema_version=CONFIG_SCHEMA_VERSION,
                h_group_level=level,
            )

    def save(self, config: PersistentBotConfig) -> None:
        if config.schema_version != CONFIG_SCHEMA_VERSION:
            raise ValueError(
                f"unsupported bot config schema version: {config.schema_version}"
            )
        self.validate_level(config.h_group_level)
        payload: dict[str, Any] = {
            "schemaVersion": CONFIG_SCHEMA_VERSION,
            "hGroupLevel": config.h_group_level,
        }
        with self.lock:
            self.path.parent.mkdir(parents=True, exist_ok=True)
            temporary = self.path.with_name(
                f".{self.path.name}.{os.getpid()}.{threading.get_ident()}.tmp"
            )
            try:
                with temporary.open("w", encoding="utf-8") as stream:
                    json.dump(payload, stream, indent=2, ensure_ascii=False)
                    stream.write("\n")
                    stream.flush()
                    os.fsync(stream.fileno())
                os.replace(temporary, self.path)
            finally:
                try:
                    temporary.unlink()
                except FileNotFoundError:
                    pass

    @staticmethod
    def validate_level(level: object) -> None:
        if level is None or level == "max":
            return
        if isinstance(level, str) and level.isdigit() and 1 <= int(level) <= 25:
            if level == str(int(level)):
                return
        raise ValueError("H-Group level must be 1 through 25, max, or null")
