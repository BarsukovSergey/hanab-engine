#!/usr/bin/env bash

set -euo pipefail

repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

if (( $# < 1 )) || [[ "${1:-}" == "--help" ]]; then
  cat <<'EOF'
Usage: bash scripts/check-scoped.sh <crate> [--] [focused cargo test args...]

Examples:
  bash scripts/check-scoped.sh hanabi-search direct_play --lib
  bash scripts/check-scoped.sh hanabi-search -- --test generic_information_set
  bash scripts/check-scoped.sh hanabi-core

The optional test arguments are passed to:
  cargo test -p <crate> --locked ...

Then the script runs repository Rust formatting and a normal crate check.
It intentionally does not run Clippy, --all-targets, broad contract suites, or
scripts/check.sh --fast.
EOF
  exit 0
fi

crate="$1"
shift
if [[ "${1:-}" == "--" ]]; then
  shift
fi

# Linked Git worktrees otherwise get separate target directories and repeatedly
# rebuild the same dependencies. Reuse the primary worktree target cache unless
# the caller has explicitly selected a CARGO_TARGET_DIR.
if [[ -z "${CARGO_TARGET_DIR:-}" ]]; then
  common_git_dir="$(git rev-parse --git-common-dir 2>/dev/null || true)"
  if [[ -n "$common_git_dir" ]]; then
    case "$common_git_dir" in
      /*|[A-Za-z]:/*) ;;
      *) common_git_dir="$repository_root/$common_git_dir" ;;
    esac
    if [[ -d "$common_git_dir" ]]; then
      common_worktree="$(cd -- "$common_git_dir/.." && pwd -P)"
      export CARGO_TARGET_DIR="$common_worktree/target"
    fi
  fi
fi

if (( $# > 0 )); then
  echo "Running focused tests for $crate..."
  cargo test -p "$crate" --locked "$@"
fi

echo 'Checking Rust formatting...'
cargo fmt --all -- --check

echo "Checking $crate..."
cargo check -p "$crate" --locked
