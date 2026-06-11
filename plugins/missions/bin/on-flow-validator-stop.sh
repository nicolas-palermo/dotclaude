#!/usr/bin/env bash
# SubagentStop for missions-flow-validator (v1 merged role). Surfaces HITL list + synthesis state.
set -u
BIN="$(dirname "$0")"
source "$BIN/lib.sh"
INPUT="$(cat)"
source "$BIN/hook-common.sh"

MILESTONE="$(jq -r '.currentMilestone // empty' "$MISSION/state.json")"
[ -z "$MILESTONE" ] && exit 0
SYN="$MISSION/validation/$MILESTONE/user-testing/synthesis.json"

if [ ! -f "$SYN" ]; then
  "$BIN/write-next-hint.sh" -m "$MISSION" "Flow validator stopped WITHOUT writing user-testing synthesis for $MILESTONE. Final message: '${LAST_MSG:0:300}'. Re-spawn missions-flow-validator."
  exit 0
fi

STATUS="$(jq -r '.status' "$SYN")"
HITL="$(jq -r '[.failedAssertions[]? | select(.reason | test("HITL"; "i")) | .id] | join(", ")' "$SYN")"
HITL_PENDING="$(jq -r '[.assertions // {} | keys[]] | length' "$MISSION/validation-state.json" 2>/dev/null || echo "?")"

if [ -n "$HITL" ]; then
  "$BIN/write-next-hint.sh" -m "$MISSION" "User-testing for $MILESTONE: $STATUS. HITL-deferred VAL-IDs: $HITL. ADVISORY: reset these to pending in validation-state.json with a note, create/extend a synthetic misc-hitl feature fulfilling them, then surface each via AskUserQuestion (state=awaiting_hitl) or continue to next milestone and batch HITL at the end."
else
  "$BIN/write-next-hint.sh" -m "$MISSION" "User-testing for $MILESTONE: $STATUS. ADVISORY: if pass and more milestones remain -> state=running, next milestone. If pass and this was the last milestone -> state=completed."
fi
exit 0
