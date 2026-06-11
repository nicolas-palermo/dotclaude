#!/usr/bin/env bash
# H-F: PostToolUse on Write of */scrutiny/synthesis.json or */user-testing/synthesis.json.
# Validates, auto-extracts fix-features from blocking issues, hints the orchestrator.
set -u
BIN="$(dirname "$0")"
source "$BIN/lib.sh"

INPUT="$(cat)"
FILE="$(printf '%s' "$INPUT" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("file_path",""))')"
[ -z "$FILE" ] || [ ! -f "$FILE" ] && exit 0

case "$FILE" in
  */scrutiny/synthesis.json)      SCHEMA="scrutiny-synthesis.schema.json"; PHASE="scrutiny" ;;
  */user-testing/synthesis.json)  SCHEMA="user-testing-synthesis.schema.json"; PHASE="user-testing" ;;
  *) exit 0 ;;
esac

ERRS="$(validate_json "$SCHEMAS_DIR/$SCHEMA" "$FILE" 2>&1)"
if [ -n "$ERRS" ]; then
  echo "Invalid $PHASE synthesis $FILE — fix and rewrite:" >&2
  echo "$ERRS" >&2
  exit 2
fi

MISSION="$(mission_from_path "$FILE")" || exit 0
MILESTONE="$(jq -r '.milestone' "$FILE")"

if [ "$PHASE" = "scrutiny" ]; then
  STATUS="$(jq -r '.status' "$FILE")"
  if [ "$STATUS" = "fail" ]; then
    K="$("$BIN/extract-fix-features.sh" -m "$MISSION" "$MILESTONE" "$FILE")"
    "$BIN/write-next-hint.sh" -m "$MISSION" "Scrutiny synthesis for $MILESTONE: FAIL. Auto-appended $K fix feature(s) at the top of features.json. ADVISORY: set state=running, spawn impl-worker on the first fix feature; scrutiny re-runs (round+1) after fixes."
  else
    "$BIN/write-next-hint.sh" -m "$MISSION" "Scrutiny synthesis for $MILESTONE: PASS. ADVISORY: spawn Agent(missions:missions-flow-validator) for user-testing of $MILESTONE."
  fi
fi
exit 0
