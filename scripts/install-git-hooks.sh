#!/usr/bin/env bash
set -euo pipefail

# Install local git hooks path to use .githooks directory
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

echo "Configuring git hooks path to .githooks" >&2
git config core.hooksPath .githooks

echo "Done. Hooks will run on next commit." >&2
