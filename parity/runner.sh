#!/usr/bin/env bash
# Plays every script in parity/PASSING and parity/PASSING_WITH_RUNNER through
# the game repository's runner, in its shared-database mode, and fails if any
# of them diverges from this repository's goldens (parity/goldens).
#
#   parity/runner.sh <server checkout> <parity binary>
#   parity/runner.sh --write <server checkout> <parity binary>
#
# With --write it writes instead the goldens of the scripts in
# parity/PASSING_WITH_RUNNER, which only the runner can play whole (it
# reloads the world file for a `reseed:` step and renders the notices a
# browser step asserts as `shown`); `parity --write parity` writes the rest.
#
# The checkout must have its test database's schema loaded
# (`RAILS_ENV=test bin/rails db:schema:load`). The runner copies that
# database into a scratch file for each script and never writes the
# database itself. Both provider keys are removed before anything runs.
set -euo pipefail

write=
if [ "${1:-}" = "--write" ]; then
  write=1
  shift
fi
here="$(cd "$(dirname "$0")" && pwd)"
server="$(cd "$1" && pwd)"
binary="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
unset OPENROUTER_API_KEY TYPESAFE_API_KEY
export RAILS_ENV=test ENGINE_DATABASE=1 ENGINE="$binary" GOLDENS="$here/goldens"

if [ -n "$write" ]; then
  scripts=$(grep -hv '^#' "$here/PASSING_WITH_RUNNER" | grep -v '^$')
else
  scripts=$(grep -hv '^#' "$here/PASSING" "$here/PASSING_WITH_RUNNER" | grep -v '^$')
fi
agreed=0
failed=()
absent=()
cd "$server"
for script in $scripts; do
  # A script this repository has ahead of the checkout has nothing to be
  # played against there yet.
  if [ ! -f "lib/engine_sweep/scripts/$script.yml" ]; then
    absent+=("$script")
    continue
  fi
  if [ -n "$write" ]; then
    task=engine:parity
  else
    task=engine:parity_diff
  fi
  if SCRIPT="$script" bin/rails "$task"; then
    agreed=$((agreed + 1))
  else
    failed+=("$script")
  fi
done

if [ -n "$write" ]; then
  echo "through the runner: wrote $agreed of $(echo "$scripts" | wc -w) listed golden(s)"
else
  echo "through the runner: $agreed of $(echo "$scripts" | wc -w) listed script(s) agree"
fi
if [ ${#absent[@]} -gt 0 ]; then
  printf 'not in the checkout yet: %s\n' "${absent[@]}"
fi
if [ ${#failed[@]} -gt 0 ]; then
  printf 'failed: %s\n' "${failed[@]}"
  exit 1
fi
