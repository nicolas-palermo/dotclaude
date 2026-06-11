#!/usr/bin/env bash
# Atomically write the orchestrator hint file (the ONLY hook->orchestrator channel; P3 fallback).
# Usage: write-next-hint.sh [-m <mission-dir>] <text>     (or text on stdin with "-")
set -eu
source "$(dirname "$0")/lib.sh"

MDIR=""
if [ "${1:-}" = "-m" ]; then MDIR="$2"; shift 2; fi
MISSION="$(resolve_mission "$MDIR")" || { echo "no active mission" >&2; exit 1; }
mkdir -p "$MISSION/hints"

if [ "${1:-}" = "-" ]; then
  atomic_write "$MISSION/hints/orchestrator-hint.txt"
else
  printf '%s\n' "$1" | atomic_write "$MISSION/hints/orchestrator-hint.txt"
fi
