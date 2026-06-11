#!/usr/bin/env bash
# Generic PostToolUse validator: validate-file.sh <schema-basename>
# stdin: hook envelope with tool_input.file_path. Exit 2 + stderr on schema violation.
set -u
source "$(dirname "$0")/lib.sh"

SCHEMA="$SCHEMAS_DIR/$1"
INPUT="$(cat)"
FILE="$(printf '%s' "$INPUT" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("file_path",""))')"
[ -z "$FILE" ] || [ ! -f "$FILE" ] && exit 0

ERRS="$(validate_json "$SCHEMA" "$FILE" 2>&1)"
if [ -n "$ERRS" ]; then
  echo "Schema violation in $FILE (schema: $1) — fix and rewrite:" >&2
  echo "$ERRS" >&2
  exit 2
fi
exit 0
