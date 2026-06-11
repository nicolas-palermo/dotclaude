#!/usr/bin/env bash
# Turn synthesis.blockingIssues into fix-features prepended to features.json.
# Usage: extract-fix-features.sh [-m <mission-dir>] <milestone> <synthesis-path>
# Prints the number of fix features created.
set -eu
BIN="$(dirname "$0")"

MDIR_ARGS=()
if [ "${1:-}" = "-m" ]; then MDIR_ARGS=(-m "$2"); shift 2; fi
MILESTONE="$1"; SYNTHESIS="$2"

COUNT="$(jq '.blockingIssues | length' "$SYNTHESIS")"
if [ "$COUNT" -eq 0 ]; then echo 0; exit 0; fi

N=0
while IFS= read -r issue; do
  N=$((N + 1))
  FEATURE="$(jq -cn --argjson i "$issue" --arg id "fix-$MILESTONE-$N" --arg m "$MILESTONE" '{
    id: $id,
    description: ("Fix blocking scrutiny issue in " + ($i.featureId // "unknown") + ": " + $i.description + (if $i.suggestedFix then " Suggested fix: " + $i.suggestedFix else "" end)),
    skillName: ($i.skillName // "impl-worker"),
    milestone: $m,
    status: "pending",
    fulfills: [],
    preconditions: [],
    expectedBehavior: [("Blocking issue resolved: " + $i.description)]
  }')"
  "$BIN/update-features.sh" "${MDIR_ARGS[@]+"${MDIR_ARGS[@]}"}" prepend-feature "$FEATURE"
done < <(jq -c '.blockingIssues[]' "$SYNTHESIS")

echo "$N"
