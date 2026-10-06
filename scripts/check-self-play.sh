#!/usr/bin/env bash
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/shell-common.sh"

repository_root="$HANABI_REPOSITORY_ROOT"
cd "$repository_root"
hanabi_add_cargo_bin_to_path
cargo test --release --locked -p hanabi-search --test self_play \
  h_group_max_self_play_200 -- --ignored --exact --nocapture
