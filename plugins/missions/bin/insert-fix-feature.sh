#!/usr/bin/env bash
# Insert a fix-feature at the TOP of features.json + log the decision (delta 4.3 post-handoff path).
# Usage: insert-fix-feature.sh [-m <mission-dir>] <json-feature-object> <narration>
set -eu
BIN="$(dirname "$0")"

MDIR_ARGS=()
if [ "${1:-}" = "-m" ]; then MDIR_ARGS=(-m "$2"); shift 2; fi
FEATURE="$1"; NARRATION="$2"

ID="$(printf '%s' "$FEATURE" | jq -re '.id')" || { echo "feature object needs .id" >&2; exit 1; }
case "$ID" in
  fix-*) ;;
  *) echo "fix feature id must start with fix- (got: $ID)" >&2; exit 1 ;;
esac

"$BIN/update-features.sh" "${MDIR_ARGS[@]+"${MDIR_ARGS[@]}"}" prepend-feature "$FEATURE"
"$BIN/append-progress.sh"  "${MDIR_ARGS[@]+"${MDIR_ARGS[@]}"}" mission_run_started "$(jq -cn --arg m "$NARRATION" '{message: $m}')"
