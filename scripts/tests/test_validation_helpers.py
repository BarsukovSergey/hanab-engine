"""Fast regression tests for cross-platform validation helpers."""

from pathlib import Path
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
ROOT = SCRIPTS.parent
sys.path.insert(0, str(SCRIPTS))

import check_lock


class ValidationHelperTests(unittest.TestCase):
    def test_python_launcher_runs_current_python_major(self) -> None:
        result = subprocess.run(
            ["bash", str(SCRIPTS / "run-python.sh"), "-c",
             "import sys; print(sys.version_info.major)"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "3")

    def test_lock_blocks_concurrent_runner_and_releases_cleanly(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            lock = Path(directory) / "check.lock"
            handle = check_lock.acquire_lock(lock)
            self.assertIsNotNone(handle)
            assert handle is not None
            blocked = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPTS / "check_lock.py"),
                    str(lock),
                    "--",
                    sys.executable,
                    "-c",
                    "raise SystemExit(0)",
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(blocked.returncode, 1)
            self.assertIn("already active", blocked.stderr)
            check_lock.release_lock(handle)

            released = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPTS / "check_lock.py"),
                    str(lock),
                    "--",
                    sys.executable,
                    "-c",
                    "raise SystemExit(0)",
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(released.returncode, 0, released.stderr)

    def test_lock_runner_preserves_child_exit_status(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPTS / "check_lock.py"),
                    str(Path(directory) / "check.lock"),
                    "--",
                    sys.executable,
                    "-c",
                    "raise SystemExit(7)",
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 7)

    def test_scoped_check_reuses_primary_worktree_target(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory) / "repo"
            linked = Path(directory) / "linked"
            (base / "scripts").mkdir(parents=True)
            shutil.copy2(SCRIPTS / "check-scoped.sh", base / "scripts" / "check-scoped.sh")
            subprocess.run(["git", "init", "-q", str(base)], check=True)
            subprocess.run(["git", "-C", str(base), "config", "user.email", "test@example.invalid"], check=True)
            subprocess.run(["git", "-C", str(base), "config", "user.name", "Validation Test"], check=True)
            subprocess.run(["git", "-C", str(base), "add", "scripts/check-scoped.sh"], check=True)
            subprocess.run(["git", "-C", str(base), "commit", "-qm", "fixture"], check=True)
            subprocess.run(
                ["git", "-C", str(base), "worktree", "add", "-q", "-b", "linked-test", str(linked)],
                check=True,
            )
            env = os.environ.copy()
            env.pop("CARGO_TARGET_DIR", None)
            result = subprocess.run(
                ["bash", str(linked / "scripts" / "check-scoped.sh"), "--print-target-dir"],
                capture_output=True,
                text=True,
                check=False,
                env=env,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(Path(result.stdout.strip()).resolve(), (base / "target").resolve())


if __name__ == "__main__":
    unittest.main()
