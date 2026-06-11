#!/usr/bin/env bash
# Locked read-modify-write of features.json. ALL feature mutations go through here.
# Usage:
#   update-features.sh [-m <mission-dir>] set-status <feature-id> <status>
#   update-features.sh [-m <mission-dir>] set-current-session <feature-id> <session-id|null>
#   update-features.sh [-m <mission-dir>] append-session <feature-id> <session-id>
#   update-features.sh [-m <mission-dir>] prepend-feature <json-object>
#   update-features.sh [-m <mission-dir>] requeue <feature-id>          # completed -> pending (delta 4.2)
set -eu
source "$(dirname "$0")/lib.sh"

MDIR=""
if [ "${1:-}" = "-m" ]; then MDIR="$2"; shift 2; fi
MISSION="$(resolve_mission "$MDIR")" || { echo "no active mission" >&2; exit 1; }
FEATURES="$MISSION/features.json"
[ -f "$FEATURES" ] || { echo "missing $FEATURES" >&2; exit 1; }

OP="$1"; shift
acquire_lock "$FEATURES.lock"

case "$OP" in
  set-status)
    jq --arg id "$1" --arg st "$2" \
      '(.features[] | select(.id == $id) | .status) = $st' "$FEATURES" | atomic_write "$FEATURES"
    ;;
  set-current-session)
    SESS="$2"
    if [ "$SESS" = "null" ]; then
      jq --arg id "$1" '(.features[] | select(.id == $id) | .currentWorkerSessionId) = null' "$FEATURES" | atomic_write "$FEATURES"
    else
      jq --arg id "$1" --arg s "$SESS" \
        '(.features[] | select(.id == $id)) |= (.currentWorkerSessionId = $s | .workerSessionIds = ((.workerSessionIds // []) + [$s] | unique))' \
        "$FEATURES" | atomic_write "$FEATURES"
    fi
    ;;
  append-session)
    jq --arg id "$1" --arg s "$2" \
      '(.features[] | select(.id == $id) | .workerSessionIds) |= ((. // []) + [$s] | unique)' "$FEATURES" | atomic_write "$FEATURES"
    ;;
  prepend-feature)
    printf '%s' "$1" | jq -e 'has("id") and has("description") and has("milestone") and has("status")' >/dev/null \
      || { echo "prepend-feature: object missing id/description/milestone/status" >&2; exit 1; }
    jq --argjson f "$1" '.features = [$f] + .features' "$FEATURES" | atomic_write "$FEATURES"
    ;;
  requeue)
    jq --arg id "$1" \
      '(.features[] | select(.id == $id)) |= (.status = "pending" | .currentWorkerSessionId = null)' \
      "$FEATURES" | atomic_write "$FEATURES"
    ;;
  *)
    echo "unknown op: $OP" >&2; exit 1
    ;;
esac

validate_json "$SCHEMAS_DIR/features.schema.json" "$FEATURES" || { echo "post-mutation schema violation in $FEATURES" >&2; exit 1; }
