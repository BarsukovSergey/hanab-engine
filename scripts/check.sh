#!/usr/bin/env bash

set -euo pipefail # Exit on errors and undefined variables.

check_started_seconds=$SECONDS
check_failed=0

# Complete the full suite even when a stage fails. Re-running the omitted tail
# manually delays feedback and makes full-validation timings misleading.
run_check() {
  local label="$1"
  shift
  local started=$SECONDS
  local status=0
  "$@" || status=$?
  printf 'CHECK TIMING: %s: %ds (exit %d)\n' \
    "$label" "$((SECONDS - started))" "$status"
  if (( status != 0 )); then
    check_failed=1
  fi
}

repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

host_uname="$(uname -s 2>/dev/null || printf 'unknown')"
case "$host_uname" in
  MINGW*|MSYS*|CYGWIN*) windows_shell=1 ;;
  *) windows_shell=0 ;;
esac

mode=full
case "${1:-}" in
  "") ;;
  --fast) mode=fast; shift ;;
  --help)
    echo 'Usage: scripts/check.sh [--fast]'
    echo 'Default: full validation. --fast: all stages except the ordinary Rust test suite.'
    echo 'Run focused Rust tests separately; fast success is not full validation.'
    exit 0 ;;
  *) echo "Unknown option: $1" >&2; exit 2 ;;
esac
if (( $# != 0 )); then
  echo 'Unexpected arguments; see --help.' >&2
  exit 2
fi
if [[ "${HANABI_CHECK_LOCKED:-0}" != 1 ]]; then
  args=()
  if [[ "$mode" == fast ]]; then args+=(--fast); fi
  exec bash scripts/run-python.sh scripts/check_lock.py target/check.lock -- \
    env HANABI_CHECK_LOCKED=1 bash scripts/check.sh "${args[@]}"
fi

if [[ "${HANABI_CHECK_TIMED:-0}" != 1 ]]; then
  args=()
  if [[ "$mode" == fast ]]; then args+=(--fast); fi
  exec env HANABI_CHECK_TIMED=1 bash scripts/run-python.sh scripts/workflow.py run \
    --label "check-$mode" -- bash scripts/check.sh "${args[@]}"
fi

# Full validations compete for the same CPU and build cache. The outer
# check_lock.py process holds a platform-native lock for this entire run, and
# the OS releases it automatically if the process exits or is killed.
check_file_ownership() {
  if (( windows_shell != 0 )); then
    echo 'Skipping Unix ownership validation under Windows Git Bash.'
    return
  fi

  local expected_owner
  local expected_group
  local repository_parent
  repository_parent="$(dirname -- "$repository_root")"
  # GNU and BSD stat use different flags. Numeric IDs also avoid depending
  # on name-service lookup. Propagate failures instead of accepting an empty
  # find result after a failed ownership query.
  if ! expected_owner="$(stat --format='%u' -- "$repository_parent" 2>/dev/null)"; then
    expected_owner="$(stat -f '%u' "$repository_parent")" || return 1
  fi
  if ! expected_group="$(stat --format='%g' -- "$repository_parent" 2>/dev/null)"; then
    expected_group="$(stat -f '%g' "$repository_parent")" || return 1
  fi

  local -a mismatched_paths=()
  while IFS= read -r -d '' path; do
    mismatched_paths+=("$path")
  done < <(
    find "$repository_root" -xdev \
      \( ! -user "$expected_owner" -o ! -group "$expected_group" \) \
      -print0
  )
  if (( ${#mismatched_paths[@]} == 0 )); then
    return
  fi

  printf >&2 \
    'Repository paths must be owned by %s:%s; found ownership mismatches:\n' \
    "$expected_owner" "$expected_group"
  printf >&2 '  %s\n' "${mismatched_paths[@]}"
  return 1
}

echo 'Checking repository file ownership...'
run_check 'Initial ownership' check_file_ownership

if ! command -v npm &> /dev/null; then
  echo 'npm is required to run this script.' >&2
  exit 1
fi

if [[ ! -x node_modules/.bin/prettier ]]; then
  echo 'Prettier was not found. Run "npm ci" in the repository root.' >&2
  exit 1
fi

echo 'Checking repository formatting with Prettier...'
run_check 'Prettier' npm run format:check

if [[ -d "$HOME/.cargo/bin" ]]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi

find_venv_tool() {
  local tool="$1"
  local candidate
  for candidate in \
    ".venv/bin/$tool" \
    ".venv/Scripts/$tool.exe" \
    ".venv/Scripts/$tool"
  do
    if [[ -f "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  done
  return 1
}

if ! venv_python="$(find_venv_tool python)" || ! venv_ty="$(find_venv_tool ty)"; then
  cat >&2 <<'EOF'
Python development dependencies are missing. Set them up with:

  python3 -m venv .venv

Then install scripts/requirements-dev.txt with the venv Python. The executable
is .venv/bin/python on POSIX and .venv/Scripts/python.exe on Windows.
EOF
  exit 1
fi

if ! rustup run 1.97.1 cargo hawk --version >/dev/null 2>&1; then
  cat >&2 <<'EOF'
cargo-hawk 0.1.10 or its Rust 1.97.1 toolchain is missing. Install the pinned
tool for the current platform before running the broad/full local checkpoint.
Do not repair this tooling as part of a scoped code task; PR CI owns this gate.
EOF
  exit 1
fi

if [[ "$mode" == full ]] && ! cargo nextest --version >/dev/null 2>&1; then
  cat >&2 <<'EOF'
cargo-nextest 0.9.143 is missing. Install that version for the current platform
before running the full local checkpoint.
EOF
  exit 1
fi

echo 'Checking Rust formatting...'
run_check 'Rust formatting' cargo fmt --all -- --check

echo 'Building all Rust targets...'
run_check 'Rust build' cargo build --workspace --all-targets --all-features --locked

echo 'Running Clippy...'
run_check 'Clippy' cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

echo 'Running all ordinary Rust tests...'
if [[ "$mode" == full ]]; then
run_check 'Rust tests' cargo nextest run \
  --workspace \
  --all-targets \
  --all-features \
  --locked \
  --no-fail-fast
else
  echo 'FAST CHECK: ordinary Rust tests skipped; run focused tests for this change.'
fi

# Nextest deliberately does not run rustdoc tests.
echo 'Running Rust documentation tests...'
run_check 'Rust documentation tests' cargo test --workspace --all-features --doc --locked

echo 'Checking Rust documentation...'
run_check 'Rust documentation' env RUSTDOCFLAGS='-D warnings' \
  cargo doc --workspace --all-features --no-deps --locked

echo 'Type-checking Python...'
run_check 'Python types' "$venv_ty" check

echo 'Compiling Python sources...'
run_check 'Python compilation' "$venv_python" -m compileall -q scripts

echo 'Running Python tests...'
run_check 'Python tests' "$venv_python" -W error::ResourceWarning -m unittest discover -s scripts/tests -v

check_cli_help() {
  "$venv_python" "$1" --help >/dev/null
}

echo 'Checking the Hanabi Live CLIs...'
run_check 'Bot CLI' check_cli_help scripts/hanabi_live_bot.py
run_check 'Parity CLI' check_cli_help scripts/hanabi_live_verify.py

echo 'Checking workspace-wide dead public code...'
run_check 'Dead public code' cargo +1.97.1 hawk check \
  --manifest-path Cargo.toml \
  --only dead-public \
  -D warnings

echo 'Rechecking repository file ownership...'
run_check 'Final ownership' check_file_ownership

check_elapsed_seconds=$((SECONDS - check_started_seconds))
if (( check_failed != 0 )); then
  printf '%s checks failed; completed in %dm %ds.\n' "$mode" \
    "$((check_elapsed_seconds / 60))" "$((check_elapsed_seconds % 60))"
  exit 1
fi
printf '%s checks passed in %dm %ds.\n' "$mode" \
  "$((check_elapsed_seconds / 60))" "$((check_elapsed_seconds % 60))"
