#!/usr/bin/env bash
# H-C: SubagentStop for missions-scrutiny-feature-reviewer. Locked aggregation counter;
# on the Nth (last) fire renders the summary and hints the orchestrator.
set -u
BIN="$(dirname "$0")"
source "$BIN/lib.sh"
INPUT="$(cat)"
source "$BIN/hook-common.sh"

MILESTONE="$(jq -r '.currentMilestone // empty' "$MISSION/state.json")"
[ -z "$MILESTONE" ] && exit 0
SCRUTINY="$MISSION/validation/$MILESTONE/scrutiny"
[ -f "$SCRUTINY/pending.jsonl" ] || exit 0

acquire_lock "$SCRUTINY/.aggregate.lock"

PLANNED="$(wc -l < "$SCRUTINY/pending.jsonl" | tr -d ' ')"
DONE="$(ls "$SCRUTINY/reviews/"*.json 2>/dev/null | wc -l | tr -d ' ')"

if [ "$DONE" -ge "$PLANNED" ]; then
  SUMMARY="$("$BIN/render-reviews-summary.sh" -m "$MISSION" "$MILESTONE" 2>/dev/null || echo "(summary failed; read reviews/ directly)")"
  {
    echo "All $DONE/$PLANNED scrutiny reviewers done for $MILESTONE. ADVISORY next: Write validation/$MILESTONE/scrutiny/synthesis.json (schema: scrutiny-synthesis). Apply/reject sharedStateObservations to library/. Summary:"
    echo "$SUMMARY"
  } | "$BIN/write-next-hint.sh" -m "$MISSION" -
fi
exit 0
