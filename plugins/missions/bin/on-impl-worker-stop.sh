#!/usr/bin/env bash
# H-A: SubagentStop for missions-impl-worker. Disk-only bookkeeping (P3 fail = no inline
# injection); briefs the orchestrator via hints/orchestrator-hint.txt.
set -u
BIN="$(dirname "$0")"
source "$BIN/lib.sh"
INPUT="$(cat)"
source "$BIN/hook-common.sh"

FEATURES="$MISSION/features.json"

# Which feature did this worker own?
FEATURE_ID="$(jq -r --arg s "$SESSION_ID" '.features[] | select(.currentWorkerSessionId == $s) | .id' "$FEATURES" | head -1)"
[ -z "$FEATURE_ID" ] && FEATURE_ID="$(jq -r --arg s "$AGENT_ID" '.features[] | select(.currentWorkerSessionId == $s) | .id' "$FEATURES" | head -1)"
[ -z "$FEATURE_ID" ] && exit 0

# Valid handoff written by this worker?
HANDOFF="$(ls -t "$MISSION/handoffs/"*"__${FEATURE_ID}__"*.json 2>/dev/null | head -1)"
RECENT_HANDOFF=""
if [ -n "$HANDOFF" ] && validate_json "$SCHEMAS_DIR/handoff.schema.json" "$HANDOFF" 2>/dev/null; then
  # Only count it if it belongs to the CURRENT session (re-runs leave older handoffs around)
  H_SESS="$(jq -r '.workerSessionId' "$HANDOFF")"
  if [ "$H_SESS" = "$SESSION_ID" ] || [ "$H_SESS" = "$AGENT_ID" ]; then RECENT_HANDOFF="$HANDOFF"; fi
fi

if [ -n "$RECENT_HANDOFF" ]; then
  SUCCESS="$(jq -r '.successState' "$RECENT_HANDOFF")"
  VPASS="$(jq -r 'if .validatorsPassed == null then empty else .validatorsPassed end' "$RECENT_HANDOFF")"
  "$BIN/update-features.sh" -m "$MISSION" set-status "$FEATURE_ID" completed
  "$BIN/update-features.sh" -m "$MISSION" set-current-session "$FEATURE_ID" null
  PAYLOAD="$(jq -cn --arg s "$SESSION_ID" --arg f "$FEATURE_ID" --arg ss "$SUCCESS" --arg vp "$VPASS" \
    '{workerSessionId: $s, featureId: $f, successState: $ss, returnToOrchestrator: true}
     + (if $vp != "" then {validatorsPassed: ($vp == "true")} else {} end)')"
  "$BIN/append-progress.sh" -m "$MISSION" worker_completed "$PAYLOAD"

  MILESTONE="$(jq -r --arg id "$FEATURE_ID" '.features[] | select(.id == $id) | .milestone' "$FEATURES")"
  REMAINING="$(jq -r --arg m "$MILESTONE" '[.features[] | select(.milestone == $m and .status != "completed")] | length' "$FEATURES")"
  if [ "$REMAINING" -eq 0 ]; then
    "$BIN/write-next-hint.sh" -m "$MISSION" "Milestone $MILESTONE impl complete. ADVISORY next step: spawn Agent(missions:missions-scrutiny-validator) for $MILESTONE. First: disposition any discoveredIssues from the last handoff (insert-fix-feature.sh or dismiss-handoff-items.sh)."
  else
    NEXT="$(jq -r --arg m "$MILESTONE" '[.features[] | select(.milestone == $m and .status == "pending")] | .[0].id // "none"' "$FEATURES")"
    "$BIN/write-next-hint.sh" -m "$MISSION" "Worker done for $FEATURE_ID ($SUCCESS). ADVISORY next eligible (ONE at a time, v1 sequential impl): $NEXT. Before spawning: disposition any discoveredIssues from the handoff (insert-fix-feature.sh or dismiss-handoff-items.sh)."
  fi
else
  # Abnormal stop: no (valid) handoff from this session
  "$BIN/update-features.sh" -m "$MISSION" set-current-session "$FEATURE_ID" null
  "$BIN/append-progress.sh" -m "$MISSION" worker_abnormal_stop \
    "$(jq -cn --arg s "$SESSION_ID" --arg f "$FEATURE_ID" '{workerSessionId: $s, featureId: $f}')"
  "$BIN/write-next-hint.sh" -m "$MISSION" "Worker ABNORMAL-STOPPED on $FEATURE_ID (no valid handoff). Final message: '${LAST_MSG:0:300}'. Re-spawn with augmented skeleton; check git log --since for partial commits; partial transcript: $TRANSCRIPT_PATH"
fi
exit 0
