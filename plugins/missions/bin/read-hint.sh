#!/usr/bin/env bash
# Read + archive the orchestrator hint. Orchestrator runs this at the top of EVERY turn.
# Prints the hint (or nothing) and moves it to hints/archive/<ts>.txt.
set -eu
source "$(dirname "$0")/lib.sh"

MDIR=""
if [ "${1:-}" = "-m" ]; then MDIR="$2"; shift 2; fi
MISSION="$(resolve_mission "$MDIR")" || exit 0
HINT="$MISSION/hints/orchestrator-hint.txt"
[ -f "$HINT" ] || exit 0

cat "$HINT"
mkdir -p "$MISSION/hints/archive"
mv "$HINT" "$MISSION/hints/archive/$(date -u +%Y%m%dT%H%M%S).txt"
