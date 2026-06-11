---
name: kill
description: Pause a mission — /missions:kill [id]. Sets state=paused; the orchestrator refuses to advance until resumed.
---

Pause the given mission (default: the active one).

1. Resolve the mission dir (argument prefix or active-mission.txt).
2. `${CLAUDE_PLUGIN_ROOT}/bin/update-state.sh -m <mission-dir> paused`
3. `${CLAUDE_PLUGIN_ROOT}/bin/append-progress.sh -m <mission-dir> mission_paused '{}'`
4. Note: in-flight subagents finish their current run; their SubagentStop hooks still do
   bookkeeping, but the orchestrator must not spawn anything new while paused. Confirm
   to the user and stop.
