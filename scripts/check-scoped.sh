#!/usr/bin/env bash

set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/shell-common.sh"

repository_root="$HANABI_REPOSITORY_ROOT"
cd "$repository_root"

configure_target_cache() {
  # Linked Git worktrees otherwise get separate target directories and
  # repeatedly rebuild the same dependencies. Reuse the primary worktree
  # target cache unless the caller explicitly selected CARGO_TARGET_DIR.
  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    return
  fi

  common_git_dir="$(git rev-parse --git-common-dir 2>/dev/null || true)"
  if [[ -z "$common_git_dir" ]]; then
    return
  fi
  case "$common_git_dir" in
    /*|[A-Za-z]:/*) ;;
    *) common_git_dir="$repository_root/$common_git_dir" ;;
  esac
  if [[ -d "$common_git_dir" ]]; then
    common_worktree="$(cd -- "$common_git_dir/.." && pwd -P)"
    export CARGO_TARGET_DIR="$common_worktree/target"
  fi
}

run_timed() {
  local label="$1"
  shift
  local started=$SECONDS
  local status=0
  "$@" || status=$?
  printf 'SCOPED TIMING: %s: %ds (exit %d)\n' "$label" "$((SECONDS - started))" "$status"
  return "$status"
}

if [[ "${1:-}" == "--help" ]] || (( $# == 0 )); then
  cat <<'EOF'
Usage: bash scripts/check-scoped.sh <crate> [--test-only|--check-only] [--] [focused cargo test args...]
       bash scripts/check-scoped.sh --print-target-dir

Examples:
  bash scripts/check-scoped.sh hanabi-search direct_play --lib
  bash scripts/check-scoped.sh hanabi-search --test-only contract_ --lib
  bash scripts/check-scoped.sh hanabi-search --check-only
  bash scripts/check-scoped.sh hanabi-search -- --test generic_information_set
  bash scripts/check-scoped.sh hanabi-core

Default mode optionally runs focused tests, then repository Rust formatting and
a normal crate check. --test-only requires focused test args and skips the final
crate check. --check-only accepts no test args and runs formatting plus the crate
check. The helper intentionally does not run Clippy, --all-targets, broad replay
suites, or scripts/check.sh --fast.
EOF
  exit 0
fi

configure_target_cache

if [[ "${1:-}" == "--print-target-dir" ]]; then
  printf '%s\n' "${CARGO_TARGET_DIR:-$repository_root/target}"
  exit 0
fi

crate="$1"
shift
mode=combined
case "${1:-}" in
  --test-only) mode=test-only; shift ;;
  --check-only) mode=check-only; shift ;;
esac
if [[ "${1:-}" == "--" ]]; then
  shift
fi

if [[ "$mode" == test-only ]] && (( $# == 0 )); then
  echo '--test-only requires focused cargo test arguments.' >&2
  exit 2
fi
if [[ "$mode" == check-only ]] && (( $# != 0 )); then
  echo '--check-only does not accept cargo test arguments.' >&2
  exit 2
fi

if [[ "$mode" != check-only ]] && (( $# > 0 )); then
  echo "Running focused tests for $crate..."
  run_timed 'Focused tests' cargo test -p "$crate" --locked "$@"
fi

echo 'Checking Rust formatting...'
run_timed 'Rust formatting' cargo fmt --all -- --check

if [[ "$mode" != test-only ]]; then
  echo "Checking $crate..."
  run_timed 'Crate check' cargo check -p "$crate" --locked
fi
