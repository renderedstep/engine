#!/usr/bin/env bash
# Plays every script in parity/PASSING and parity/PASSING_WITH_RUNNER through
# the Ruby engine's own runner, in its shared-database mode, and fails if any
# of them diverges from the goldens there.
#
#   parity/runner.sh <text-adventure checkout> <parity binary>
#
# The checkout must have its test database's schema loaded
# (`RAILS_ENV=test bin/rails db:schema:load`). The runner copies that
# database into a scratch file for each script and never writes the
# database itself. Both provider keys are removed before anything runs.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
ruby_engine="$(cd "$1" && pwd)"
binary="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
unset OPENROUTER_API_KEY TYPESAFE_API_KEY
export RAILS_ENV=test ENGINE_DATABASE=1 ENGINE="$binary"

scripts=$(grep -hv '^#' "$here/PASSING" "$here/PASSING_WITH_RUNNER" | grep -v '^$')
agreed=0
failed=()
cd "$ruby_engine"
for script in $scripts; do
  if SCRIPT="$script" bin/rails engine:parity_diff; then
    agreed=$((agreed + 1))
  else
    failed+=("$script")
  fi
done

echo "through the runner: $agreed of $(echo "$scripts" | wc -w) listed script(s) agree"
if [ ${#failed[@]} -gt 0 ]; then
  printf 'diverged: %s\n' "${failed[@]}"
  exit 1
fi
