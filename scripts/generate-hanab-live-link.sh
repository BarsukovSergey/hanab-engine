#!/usr/bin/env bash
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/shell-common.sh"

repository_root="$HANABI_REPOSITORY_ROOT"
hanabi_add_cargo_bin_to_path
# Keep the caller's working directory so relative replay paths remain valid.
# Cargo status goes to stderr; stdout contains one verified Markdown link.
exec cargo run --quiet --locked --manifest-path "$repository_root/Cargo.toml" \
  -p hanabi-cli --bin hanabi-engine -- replay-link "$@" --markdown
