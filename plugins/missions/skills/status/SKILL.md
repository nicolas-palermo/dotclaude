---
name: status
description: Show mission status — /missions:status [id]. One compact disk-truth snapshot.
---

Print the status snapshot for the given mission (default: active):

```bash
${CLAUDE_PLUGIN_ROOT}/bin/render-status.sh [mission-dir]
```

Then add 2-3 sentences of interpretation: what phase it's in, what's blocking, and the
likely next step. Do NOT advance the state machine from this skill.
