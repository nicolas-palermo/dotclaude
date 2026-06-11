#!/usr/bin/env bash
# Append a typed event to progress_log.jsonl.
# Usage: append-progress.sh [-m <mission-dir>] <type> [json-payload]
# The payload object is merged into {timestamp, type}.
set -eu
source "$(dirname "$0")/lib.sh"

MDIR=""
if [ "${1:-}" = "-m" ]; then MDIR="$2"; shift 2; fi
MISSION="$(resolve_mission "$MDIR")" || { echo "no active mission" >&2; exit 1; }
LOG="$MISSION/progress_log.jsonl"

TYPE="$1"; PAYLOAD="${2:-{\}}"
EVENT="$(jq -cn --arg ts "$(now_iso)" --arg t "$TYPE" --argjson p "$PAYLOAD" '{timestamp: $ts, type: $t} + $p')"

python3 - "$SCHEMAS_DIR/progress-event.schema.json" "$EVENT" <<'EOF'
import json, sys
from jsonschema import Draft202012Validator
errs = list(Draft202012Validator(json.load(open(sys.argv[1]))).iter_errors(json.loads(sys.argv[2])))
for e in errs[:5]:
    print(f"{'/'.join(map(str, e.path)) or '<root>'}: {e.message[:160]}", file=sys.stderr)
sys.exit(1 if errs else 0)
EOF

acquire_lock "$LOG.lock"
echo "$EVENT" >> "$LOG"
