#!/usr/bin/env bash

set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/shell-common.sh"

if hanabi_is_windows_shell; then
  candidates=(python python3)
else
  candidates=(python3 python)
fi

for candidate in "${candidates[@]}"; do
  if command -v "$candidate" >/dev/null 2>&1; then
    exec "$candidate" "$@"
  fi
done

echo 'Python 3 is required but neither python3 nor python is available.' >&2
exit 127
