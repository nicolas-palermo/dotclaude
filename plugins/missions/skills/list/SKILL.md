---
name: list
description: List this project's missions — /missions:list.
---

List all missions in this project:

```bash
ACTIVE="$(cat ./.claude/missions/active-mission.txt 2>/dev/null)"
for s in ./.claude/missions/*/state.json; do
  [ -f "$s" ] || continue
  id="$(basename "$(dirname "$s")")"
  mark=""; [ "$id" = "$ACTIVE" ] && mark=" (active)"
  jq -r --arg id "$id" --arg mark "$mark" '"- \($id[0:8])\($mark): \(.state) — updated \(.updatedAt)"' "$s"
done
```

Show the objective line from each mission's mission.md alongside. If none exist, say so
and point at /missions:start.
