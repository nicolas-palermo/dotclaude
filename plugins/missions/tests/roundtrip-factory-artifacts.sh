#!/usr/bin/env bash
# Round-trip validation: every schema must accept the real Factory mission artifacts.
# Usage: roundtrip-factory-artifacts.sh [mission-dir]
set -u
MISSION="${1:-$HOME/.factory/missions/0e2031d2-e466-4e2f-8ba5-0b09d3723ba5}"
SCHEMAS="$(cd "$(dirname "$0")/../schemas" && pwd)"
FAIL=0

check() { # schema file
  local schema="$1" file="$2"
  if python3 - "$SCHEMAS/$schema" "$file" <<'EOF'
import json, sys
from jsonschema import Draft202012Validator
schema = json.load(open(sys.argv[1]))
data = json.load(open(sys.argv[2]))
errs = list(Draft202012Validator(schema).iter_errors(data))
for e in errs[:5]:
    print(f"  {'/'.join(map(str, e.path)) or '<root>'}: {e.message[:140]}", file=sys.stderr)
sys.exit(1 if errs else 0)
EOF
  then echo "PASS  $schema <- ${file#$MISSION/}"
  else echo "FAIL  $schema <- ${file#$MISSION/}"; FAIL=1
  fi
}

check_lines() { # schema jsonl-file
  local schema="$1" file="$2"
  if python3 - "$SCHEMAS/$schema" "$file" <<'EOF'
import json, sys
from jsonschema import Draft202012Validator
v = Draft202012Validator(json.load(open(sys.argv[1])))
bad = 0
for i, line in enumerate(open(sys.argv[2]), 1):
    line = line.strip()
    if not line: continue
    errs = list(v.iter_errors(json.loads(line)))
    if errs:
        bad += 1
        if bad <= 3:
            print(f"  line {i}: {errs[0].message[:140]}", file=sys.stderr)
sys.exit(1 if bad else 0)
EOF
  then echo "PASS  $schema <- ${file#$MISSION/} (all lines)"
  else echo "FAIL  $schema <- ${file#$MISSION/}"; FAIL=1
  fi
}

check state.schema.json            "$MISSION/state.json"
check features.schema.json         "$MISSION/features.json"
check validation-state.schema.json "$MISSION/validation-state.json"
for f in "$MISSION"/handoffs/*.json;                          do check handoff.schema.json "$f"; done
for f in "$MISSION"/validation/*/scrutiny/reviews/*.json;     do check review.schema.json "$f"; done
for f in "$MISSION"/validation/*/scrutiny/synthesis.json;     do check scrutiny-synthesis.schema.json "$f"; done
for f in "$MISSION"/validation/*/user-testing/synthesis.json; do check user-testing-synthesis.schema.json "$f"; done
for f in "$MISSION"/validation/*/user-testing/flows/*.json;   do check flow-report.schema.json "$f"; done
check_lines progress-event.schema.json "$MISSION/progress_log.jsonl"

exit $FAIL
