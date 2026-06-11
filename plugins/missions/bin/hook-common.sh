#!/usr/bin/env bash
# Shared prologue for SubagentStop hooks. Source AFTER reading stdin into $INPUT.
# Sets: AGENT_ID AGENT_TYPE SESSION_ID TRANSCRIPT_PATH LAST_MSG CWD MISSION
# Exits 0 early on stop_hook_active (fires ~10x otherwise — Phase 0 sub-finding) or no mission.

ACTIVE="$(printf '%s' "$INPUT" | jq -r '.stop_hook_active // false')"
if [ "$ACTIVE" = "true" ]; then exit 0; fi

AGENT_ID="$(printf '%s' "$INPUT" | jq -r '.agent_id // empty')"
AGENT_TYPE="$(printf '%s' "$INPUT" | jq -r '.agent_type // empty')"
SESSION_ID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty')"
TRANSCRIPT_PATH="$(printf '%s' "$INPUT" | jq -r '.agent_transcript_path // empty')"
LAST_MSG="$(printf '%s' "$INPUT" | jq -r '.last_assistant_message // empty')"
CWD="$(printf '%s' "$INPUT" | jq -r '.cwd // empty')"

[ -n "$CWD" ] && export CLAUDE_PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$CWD}"
MISSION="$(resolve_mission "")" || exit 0
[ -d "$MISSION" ] || exit 0
