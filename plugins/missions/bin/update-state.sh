#!/usr/bin/env bash
# Locked update of state.json. Usage: update-state.sh [-m <mission-dir>] <new-state> [currentMilestone]
set -eu
source "$(dirname "$0")/lib.sh"

MDIR=""
if [ "${1:-}" = "-m" ]; then MDIR="$2"; shift 2; fi
MISSION="$(resolve_mission "$MDIR")" || { echo "no active mission" >&2; exit 1; }
STATE="$MISSION/state.json"
[ -f "$STATE" ] || { echo "missing $STATE" >&2; exit 1; }

NEW="$1"; MILESTONE="${2:-}"
acquire_lock "$STATE.lock"

NOW="$(now_iso)"
if [ -n "$MILESTONE" ]; then
  jq --arg st "$NEW" --arg now "$NOW" --arg m "$MILESTONE" \
    '.state = $st | .updatedAt = $now | .currentMilestone = $m' "$STATE" | atomic_write "$STATE"
else
  jq --arg st "$NEW" --arg now "$NOW" '.state = $st | .updatedAt = $now' "$STATE" | atomic_write "$STATE"
fi

validate_json "$SCHEMAS_DIR/state.schema.json" "$STATE" || { echo "post-mutation schema violation in $STATE" >&2; exit 1; }
