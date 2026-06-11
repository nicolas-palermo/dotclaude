#!/usr/bin/env bash
# Post-mission assertions for the FizzBuzz E2E. Usage: assertions.sh <project-dir>
set -u
PROJ="$1"
PLUGIN="$(cd "$(dirname "$0")/../.." && pwd)"
FAIL=0
ok()   { echo "PASS  $1"; }
bad()  { echo "FAIL  $1"; FAIL=1; }

UUID="$(cat "$PROJ/.claude/missions/active-mission.txt" 2>/dev/null)" || { bad "active-mission.txt exists"; exit 1; }
M="$PROJ/.claude/missions/$UUID"

[ "$(jq -r '.state' "$M/state.json")" = "completed" ] && ok "state == completed" || bad "state == completed (got $(jq -r '.state' "$M/state.json"))"

INCOMPLETE="$(jq '[.features[] | select(.status != "completed")] | length' "$M/features.json")"
[ "$INCOMPLETE" -eq 0 ] && ok "all features completed" || bad "all features completed ($INCOMPLETE pending)"

FIX="$(jq '[.features[] | select(.id | startswith("fix-"))] | length' "$M/features.json")"
[ "$FIX" -ge 1 ] && ok "fix loop exercised ($FIX fix features)" || bad "at least one fix- feature (seeded bug should trigger scrutiny fail)"

NOTPASSED="$(jq '[.assertions[] | select(.status != "passed")] | length' "$M/validation-state.json")"
[ "$NOTPASSED" -eq 0 ] && ok "all VAL-IDs passed" || bad "all VAL-IDs passed ($NOTPASSED not passed)"

NOMILESTONE="$(jq '[.assertions[] | select(.status == "passed" and (.validatedAtMilestone // "") == "")] | length' "$M/validation-state.json")"
[ "$NOMILESTONE" -eq 0 ] && ok "passed VAL-IDs carry validatedAtMilestone" || bad "validatedAtMilestone present on all passed"

HANDOFF_BAD=0
for f in "$M"/handoffs/*.json; do
  python3 -c "
import json, sys
from jsonschema import Draft202012Validator
errs = list(Draft202012Validator(json.load(open('$PLUGIN/schemas/handoff.schema.json'))).iter_errors(json.load(open('$f'))))
sys.exit(1 if errs else 0)" || HANDOFF_BAD=$((HANDOFF_BAD + 1))
done
[ "$HANDOFF_BAD" -eq 0 ] && ok "all handoffs re-validate" || bad "$HANDOFF_BAD handoffs fail schema"

python3 -c "
import json, sys
[json.loads(l) for l in open('$M/progress_log.jsonl') if l.strip()]
" && ok "progress_log all lines parse" || bad "progress_log parse"

TYPES="$(jq -rs 'map(.type) | join(",")' "$M/progress_log.jsonl")"
case "$TYPES" in
  *mission_accepted*mission_run_started*worker_started*worker_completed*) ok "event ordering sane" ;;
  *) bad "event ordering (got: ${TYPES:0:120}...)" ;;
esac

VT="$(grep -c milestone_validation_triggered "$M/progress_log.jsonl" || true)"
[ "$VT" -ge 2 ] && ok ">=2 milestone validations" || bad ">=2 milestone_validation_triggered (got $VT)"

EV="$(find "$M/evidence" -type f -size +0c 2>/dev/null | wc -l | tr -d ' ')"
[ "$EV" -ge 1 ] && ok "evidence files exist ($EV)" || bad "non-empty evidence files"

exit $FAIL
