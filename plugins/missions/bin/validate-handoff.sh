#!/usr/bin/env bash
# PostToolUse hook: validate a worker handoff JSON against schema + VAL-ID cross-check.
# stdin: hook envelope. Exit 2 + stderr feeds the error back to the worker.
set -u
source "$(dirname "$0")/lib.sh"

INPUT="$(cat)"
FILE="$(printf '%s' "$INPUT" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("file_path",""))')"
[ -z "$FILE" ] || [ ! -f "$FILE" ] && exit 0

ERRS="$(validate_json "$SCHEMAS_DIR/handoff.schema.json" "$FILE" 2>&1)"
if [ -n "$ERRS" ]; then
  echo "Invalid handoff $FILE — fix and rewrite the file:" >&2
  echo "$ERRS" >&2
  exit 2
fi

# Cross-check: every tests.added[].cases[].verifies VAL-ID must exist in validation-contract.md
MISSION="$(mission_from_path "$FILE")" || exit 0
CONTRACT="$MISSION/validation-contract.md"
if [ -f "$CONTRACT" ]; then
  MISSING="$(python3 - "$FILE" "$CONTRACT" <<'EOF'
import json, re, sys
h = json.load(open(sys.argv[1]))
contract = open(sys.argv[2]).read()
missing = []
for t in h.get("handoff", {}).get("tests", {}).get("added", []):
    for c in t.get("cases", []):
        v = c.get("verifies", "")
        for val_id in re.findall(r"VAL-[A-Z]+-\d{3,}", v):
            if val_id not in contract:
                missing.append(val_id)
print("\n".join(sorted(set(missing))))
EOF
)"
  if [ -n "$MISSING" ]; then
    echo "Handoff references VAL-IDs not present in validation-contract.md:" >&2
    echo "$MISSING" >&2
    exit 2
  fi
fi
exit 0
