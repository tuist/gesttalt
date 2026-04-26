#!/usr/bin/env bash
set -euo pipefail

WORKTREE_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

if git -C "${WORKTREE_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  git -C "${WORKTREE_ROOT}" config extensions.worktreeConfig true
fi

