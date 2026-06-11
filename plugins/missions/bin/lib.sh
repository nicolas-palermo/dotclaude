#!/usr/bin/env bash
# Shared helpers for missions bin/ scripts. Source this; do not execute.
# macOS has no flock(1); locking uses mkdir (atomic on POSIX) with timeout.

MISSIONS_LOCK_TIMEOUT="${MISSIONS_LOCK_TIMEOUT:-15}"

# acquire_lock <lockdir>  — spinlock; sets trap to release on exit
acquire_lock() {
  local lockdir="$1" waited=0
  while ! mkdir "$lockdir" 2>/dev/null; do
    sleep 0.2
    waited=$((waited + 1))
    if [ "$waited" -ge $((MISSIONS_LOCK_TIMEOUT * 5)) ]; then
      # Stale lock older than 60s gets broken
      if [ -n "$(find "$lockdir" -maxdepth 0 -mmin +1 2>/dev/null)" ]; then
        rm -rf "$lockdir"
        continue
      fi
      echo "lock timeout: $lockdir" >&2
      return 1
    fi
  done
  # shellcheck disable=SC2064
  trap "rm -rf '$lockdir'" EXIT INT TERM
}

# atomic_write <target>  — reads stdin, writes tmp, mv into place
atomic_write() {
  local target="$1" tmp
  tmp="$(mktemp "${target}.XXXXXX")"
  cat > "$tmp"
  mv "$tmp" "$target"
}

# resolve_mission [explicit-dir]  — echoes the active mission dir
# Order: $1 if given, $MISSIONS_DIR env, ./.claude/missions/active-mission.txt
resolve_mission() {
  if [ -n "${1:-}" ]; then echo "$1"; return 0; fi
  if [ -n "${MISSIONS_DIR:-}" ]; then echo "$MISSIONS_DIR"; return 0; fi
  local base="${CLAUDE_PROJECT_DIR:-$PWD}"
  local ptr="$base/.claude/missions/active-mission.txt"
  if [ -f "$ptr" ]; then
    echo "$base/.claude/missions/$(cat "$ptr" | tr -d '[:space:]')"
    return 0
  fi
  return 1
}

# mission_from_path <file-path>  — derives the mission dir from a path like
# <anything>/.claude/missions/<uuid>/<...>  (also matches Factory-style dirs for tests)
mission_from_path() {
  local p="$1"
  case "$p" in
    */missions/*)
      echo "$(echo "$p" | sed -E 's#(.*/missions/[^/]+)/.*#\1#')"
      ;;
    *) return 1 ;;
  esac
}

# now_iso  — UTC ISO-8601 with milliseconds
now_iso() {
  python3 -c "from datetime import datetime, timezone; print(datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S.%f')[:-3] + 'Z')"
}

# validate_json <schema-path> <file>  — exits 0/1, errors on stderr
validate_json() {
  python3 - "$1" "$2" <<'EOF'
import json, sys
from jsonschema import Draft202012Validator
try:
    data = json.load(open(sys.argv[2]))
except json.JSONDecodeError as e:
    print(f"invalid JSON: {e}", file=sys.stderr)
    sys.exit(1)
errs = list(Draft202012Validator(json.load(open(sys.argv[1]))).iter_errors(data))
for e in errs[:10]:
    path = '/'.join(map(str, e.path)) or '<root>'
    print(f"{path}: {e.message[:200]}", file=sys.stderr)
sys.exit(1 if errs else 0)
EOF
}

SCHEMAS_DIR="${CLAUDE_PLUGIN_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}/schemas"
