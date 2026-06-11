#!/usr/bin/env bash
# Record explicit dispositions for handoff items not extracted to fix-features (delta 4.4).
# Usage: dismiss-handoff-items.sh [-m <mission-dir>] <json-dismissals-array>
# Each item: {type: "discovered_issue"|"incomplete_work", sourceFeatureId, summary, justification}
set -eu
BIN="$(dirname "$0")"

MDIR_ARGS=()
if [ "${1:-}" = "-m" ]; then MDIR_ARGS=(-m "$2"); shift 2; fi

printf '%s' "$1" | jq -e 'type == "array" and length > 0' >/dev/null \
  || { echo "argument must be a non-empty JSON array of dismissals" >&2; exit 1; }

"$BIN/append-progress.sh" "${MDIR_ARGS[@]+"${MDIR_ARGS[@]}"}" handoff_items_dismissed \
  "$(jq -cn --argjson d "$1" '{dismissals: $d}')"
