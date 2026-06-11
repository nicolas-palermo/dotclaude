#!/usr/bin/env bash
# Summarize all reviews of a milestone phase into a compact table (orchestrator never
# bulk-reads review JSONs). Usage: render-reviews-summary.sh [-m <mission>] <milestone>
set -eu
source "$(dirname "$0")/lib.sh"

MDIR=""
if [ "${1:-}" = "-m" ]; then MDIR="$2"; shift 2; fi
MISSION="$(resolve_mission "$MDIR")" || { echo "no active mission" >&2; exit 1; }
MILESTONE="$1"
DIR="$MISSION/validation/$MILESTONE/scrutiny/reviews"
[ -d "$DIR" ] || { echo "no reviews at $DIR" >&2; exit 1; }

echo "| feature | status | blocking | non-blocking | summary |"
echo "|---|---|---|---|---|"
for f in "$DIR"/*.json; do
  jq -r '"| \(.featureId) | \(.status) | \([.codeReview.issues[] | select(.severity == "blocking")] | length) | \([.codeReview.issues[] | select(.severity == "non_blocking")] | length) | \(.summary[0:140] | gsub("\n"; " ")) |"' "$f"
done
echo
echo "Shared-state observations (validator merges; reviewers never write library/):"
for f in "$DIR"/*.json; do
  jq -r '.sharedStateObservations[]? | "- [\(.area)] \(.observation[0:140])"' "$f"
done
