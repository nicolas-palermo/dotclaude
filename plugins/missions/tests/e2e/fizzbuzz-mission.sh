#!/usr/bin/env bash
# End-to-end FizzBuzz mission. Long-running (spawns a real headless Claude session) —
# run manually, not in CI. Requires: claude CLI, the missions plugin installed (or
# CLAUDE_PLUGIN_ROOT pointing at the repo checkout), jq, python3+jsonschema.
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"

PROJ="$(mktemp -d)/fizzbuzz-proj"
mkdir -p "$PROJ"
cd "$PROJ"
git init -q && git commit -q --allow-empty -m init

mkdir -p .claude
cat > .claude/settings.json <<'EOF'
{ "agent": "missions:missions-orchestrator" }
EOF

# Headless interview: P8 fallback — orchestrator reads answers from file
export MISSIONS_HEADLESS=1
mkdir -p .claude/missions
# interview-answers.json is staged into the mission dir by the start skill prompt below

OBJECTIVE="Build a Python CLI fizzbuzz that takes --from N --to M, prints lines per FizzBuzz rules, with both pytest unit tests and a subprocess integration test. NOTE FOR PLANNING (headless): read interview answers from $HERE/fizzbuzz-interview-answers.json and copy it to <mission>/planning/interview-answers.json. Seed hint for m1: a common first implementation checks i%3==0 or i%5==0 before i%15 — make the first worker likely to hit this and let scrutiny catch it."

claude -p "/missions:start \"$OBJECTIVE\"" \
  --output-format stream-json \
  --max-turns 300 \
  2>&1 | tee "$PROJ/mission-run.log" | grep -E '"type":"(result|system)"' || true

echo
echo "=== assertions ==="
"$HERE/assertions.sh" "$PROJ"
RC=$?
echo "project: $PROJ"
exit $RC
