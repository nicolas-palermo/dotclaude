#!/usr/bin/env bash
# H-G: SessionStart (resume|compact). Injects a recovery snapshot via additionalContext
# (supported for SessionStart per docs — unlike SubagentStop).
set -u
BIN="$(dirname "$0")"
source "$BIN/lib.sh"

INPUT="$(cat)"
CWD="$(printf '%s' "$INPUT" | jq -r '.cwd // empty')"
[ -n "$CWD" ] && export CLAUDE_PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$CWD}"

MISSION="$(resolve_mission "")" || exit 0   # non-mission project: silent no-op
[ -f "$MISSION/state.json" ] || exit 0

SNAPSHOT="$("$BIN/render-status.sh" "$MISSION" 2>/dev/null | head -c 6000)"
jq -cn --arg ctx "ACTIVE MISSION RECOVERY SNAPSHOT (disk is source of truth; rerun bin/render-status.sh anytime):
$SNAPSHOT" '{hookSpecificOutput: {hookEventName: "SessionStart", additionalContext: $ctx}}'
exit 0
