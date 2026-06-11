---
name: resume
description: Resume a mission — /missions:resume <id-or-prefix>. Points active-mission.txt at it and re-enters the state machine from disk.
---

Resume the mission given by the argument (uuid or unambiguous prefix; if no argument and
exactly one mission exists, use it — otherwise list candidates and ask).

1. Resolve the id under `./.claude/missions/`; update `active-mission.txt`.
2. If state.json says `paused`, set it back to its pre-pause state with
   `${CLAUDE_PLUGIN_ROOT}/bin/update-state.sh running` and log
   `bin/append-progress.sh mission_resumed '{}'`.
3. Print `${CLAUDE_PLUGIN_ROOT}/bin/render-status.sh` output.
4. Continue the orchestrator state machine from whatever state.json says — run the
   recovery protocol first (read-hint, state, features, progress tail, pending fan-outs).
