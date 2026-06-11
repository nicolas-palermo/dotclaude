#!/usr/bin/env bash
# One ~2-3KB Markdown snapshot of the mission. Used by /missions:status and recovery.
set -eu
source "$(dirname "$0")/lib.sh"

MISSION="$(resolve_mission "${1:-}")" || { echo "no active mission" >&2; exit 1; }

echo "# Mission status — $(jq -r '.missionId' "$MISSION/state.json")"
jq -r '"- state: \(.state)\n- workingDirectory: \(.workingDirectory)\n- updatedAt: \(.updatedAt)"' "$MISSION/state.json"
echo
echo "## Features by milestone"
jq -r '.features | group_by(.milestone)[] |
  "- **\(.[0].milestone)**: " + (group_by(.status) | map("\(.[0].status)=\(length)") | join(", "))' "$MISSION/features.json"
echo
echo "## In progress / pending"
jq -r '.features[] | select(.status != "completed") | "- \(.id) [\(.status)]" + (if .currentWorkerSessionId then " worker=\(.currentWorkerSessionId)" else "" end)' "$MISSION/features.json"
echo
if [ -f "$MISSION/validation-state.json" ]; then
  echo "## Validation"
  jq -r '.assertions | to_entries | group_by(.value.status) | map("- \(.[0].value.status): \(length)") | join("\n")' "$MISSION/validation-state.json"
  echo
fi
echo "## Recent events"
tail -n 8 "$MISSION/progress_log.jsonl" 2>/dev/null | jq -r '"- \(.timestamp) \(.type)" + (if .featureId then " \(.featureId)" else "" end) + (if .message then ": \(.message[0:100])" else "" end)'
if [ -f "$MISSION/hints/orchestrator-hint.txt" ]; then
  echo
  echo "## Pending hint"
  cat "$MISSION/hints/orchestrator-hint.txt"
fi
