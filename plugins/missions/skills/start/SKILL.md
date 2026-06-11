---
name: start
description: Start a new mission — /missions:start "<objective>". Creates the per-project mission control plane and enters the planning interview.
---

Start a new mission with the objective given in the arguments (ask for one via
AskUserQuestion if missing). Steps — run them now:

1. **First-run checks** (warn, don't block):
   - The user's `.claude/settings.json` (project or user scope) should set
     `"agent": "missions:missions-orchestrator"` for mission sessions. If absent, print
     the exact line to add and continue.
   - `.claude/missions/` should be git-ignored (interview answers can be sensitive; the
     dir bloats commits). If not covered by `.gitignore`/`.git/info/exclude`, warn and
     offer to append.
2. **Mint the mission**: `UUID="$(uuidgen | tr 'A-Z' 'a-z')"`. Create
   `./.claude/missions/$UUID/` plus subdirs `handoffs evidence validation library skills
   planning hints`. Copy every `${CLAUDE_PLUGIN_ROOT}/templates/new-mission/*.tpl` into
   the mission dir (strip the `.tpl` suffix) replacing `{{MISSION_SHORT_ID}}` (first 8
   uuid chars), `{{NOW}}` (UTC ISO-8601), `{{WORKING_DIRECTORY}}` (the project root, or
   what the user specified), `{{OBJECTIVE}}` / `{{OBJECTIVE_TITLE}}`.
   Write `$PWD` git branch + working dir to `working_directory.txt`.
3. **Activate**: write the uuid to `./.claude/missions/active-mission.txt`.
4. **Log**: `${CLAUDE_PLUGIN_ROOT}/bin/append-progress.sh mission_run_started
   '{"message":"Mission created: <objective summary>. Starting planning interview."}'`
5. Begin the **planning interview** per the missions-orchestrator state machine
   (state.json already says planning_interview). If the current session agent is NOT
   missions-orchestrator, tell the user to restart with it and stop here.
