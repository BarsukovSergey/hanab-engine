#!/usr/bin/env bash

set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/shell-common.sh"

repository_root="$HANABI_REPOSITORY_ROOT"
cd "$repository_root"
hanabi_add_cargo_bin_to_path

echo 'Running the exhaustive H-Group profile matrix...'
cargo test --locked -p hanabi-search profile_rollout_ -- --ignored

echo 'All exhaustive checks passed.'
