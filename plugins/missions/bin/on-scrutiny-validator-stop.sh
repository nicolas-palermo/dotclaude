#!/usr/bin/env bash
# H-B: SubagentStop for missions-scrutiny-validator. Reads the fan-out plan it wrote,
# stages pending.jsonl (durability rule 2) and hints the orchestrator.
set -u
BIN="$(dirname "$0")"
source "$BIN/lib.sh"
INPUT="$(cat)"
source "$BIN/hook-common.sh"

MILESTONE="$(jq -r '.currentMilestone // empty' "$MISSION/state.json")"
[ -z "$MILESTONE" ] && MILESTONE="$(ls -t "$MISSION/validation/" 2>/dev/null | head -1)"
PLAN="$MISSION/validation/$MILESTONE/scrutiny/plan.json"

if [ ! -f "$PLAN" ]; then
  "$BIN/write-next-hint.sh" -m "$MISSION" "Scrutiny validator stopped WITHOUT writing plan.json for $MILESTONE. Final message: '${LAST_MSG:0:300}'. Re-spawn missions-scrutiny-validator."
  exit 0
fi

N="$(jq '.features | length' "$PLAN")"
PENDING="$MISSION/validation/$MILESTONE/scrutiny/pending.jsonl"
jq -c '.features[] | {featureId}' "$PLAN" > "$PENDING"

"$BIN/write-next-hint.sh" -m "$MISSION" "Scrutiny plan ready for $MILESTONE: $N features to review (see validation/$MILESTONE/scrutiny/plan.json). ADVISORY next step: spawn $N parallel Agent(missions:missions-scrutiny-feature-reviewer) calls IN ONE TURN, one per featureId in the plan. pending.jsonl staged."
exit 0
