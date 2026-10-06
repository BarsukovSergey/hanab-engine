#!/usr/bin/env bash

set -euo pipefail

host_uname="$(uname -s 2>/dev/null || printf 'unknown')"
case "$host_uname" in
  MINGW*|MSYS*|CYGWIN*)
    candidates=(python python3)
    ;;
  *)
    candidates=(python3 python)
    ;;
esac

for candidate in "${candidates[@]}"; do
  if command -v "$candidate" >/dev/null 2>&1; then
    exec "$candidate" "$@"
  fi
done

echo 'Python 3 is required but neither python3 nor python is available.' >&2
exit 127
