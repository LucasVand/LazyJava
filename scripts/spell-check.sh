#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Requires codespell to be installed (e.g. `brew install codespell` or
# `python3 -m pip install --user codespell`). Matches the `spell-check` CI
# workflow in `.github/workflows/spell-check.yml`.
exec codespell --toml codespell.toml src tests