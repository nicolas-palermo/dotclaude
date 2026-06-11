#!/usr/bin/env bash
# Locked update of validation-state.json.
# Usage: update-validation-state.sh [-m <mission-dir>] <VAL-ID> <status> [milestone] [note]
set -eu
source "$(dirname "$0")/lib.sh"

MDIR=""
if [ "${1:-}" = "-m" ]; then MDIR="$2"; shift 2; fi
MISSION="$(resolve_mission "$MDIR")" || { echo "no active mission" >&2; exit 1; }
VSTATE="$MISSION/validation-state.json"
[ -f "$VSTATE" ] || { echo "missing $VSTATE" >&2; exit 1; }

VAL_ID="$1"; STATUS="$2"; MILESTONE="${3:-}"; NOTE="${4:-}"
acquire_lock "$VSTATE.lock"

jq --arg id "$VAL_ID" --arg st "$STATUS" --arg m "$MILESTONE" --arg note "$NOTE" '
  .assertions[$id] = (
    {status: $st}
    + (if $m    != "" then {validatedAtMilestone: $m} else {} end)
    + (if $note != "" then {note: $note} else {} end)
  )' "$VSTATE" | atomic_write "$VSTATE"

validate_json "$SCHEMAS_DIR/validation-state.schema.json" "$VSTATE" || { echo "post-mutation schema violation" >&2; exit 1; }
