#!/usr/bin/env bash

# Shared shell environment helpers for repository scripts.
#
# This file is sourced by entrypoints; it does not enable shell options or
# change the caller's working directory.
HANABI_REPOSITORY_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"

hanabi_is_windows_shell() {
  local host_uname
  host_uname="$(uname -s 2>/dev/null || printf 'unknown')"
  case "$host_uname" in
    MINGW*|MSYS*|CYGWIN*) return 0 ;;
    *) return 1 ;;
  esac
}

hanabi_add_cargo_bin_to_path() {
  if [[ -n "${HOME:-}" && -d "$HOME/.cargo/bin" ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
  fi
}

hanabi_find_venv_tool() {
  local tool="$1"
  local candidate
  for candidate in     ".venv/bin/$tool"     ".venv/Scripts/$tool.exe"     ".venv/Scripts/$tool"
  do
    if [[ -f "$HANABI_REPOSITORY_ROOT/$candidate" ]]; then
      printf '%s\n' "$HANABI_REPOSITORY_ROOT/$candidate"
      return
    fi
  done
  return 1
}
