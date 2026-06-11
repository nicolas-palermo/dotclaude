# Phase 0 — Mechanics spike results

Probes P1–P9 were executed during the remote Ultraplan session (2026-06-10) against
Claude Code's plugin/hook machinery. Outcomes below are baked into the plan's Findings
as [PROVED] / [FALLBACK ENGAGED]. Re-verify on this machine if Claude Code is upgraded.

| # | Capability | Result | Consequence |
|---|---|---|---|
| P1 | `PostToolUse` `if:` filter + exit-2 stderr → model | **PASS** | Schema validators use exit 2 + stderr. Note: PostToolUse cannot block the write itself — the bad file IS written; the worker sees the error next turn and overwrites. |
| P2 | `SubagentStop` fires, per-agent matcher, rich payload | **PASS** | Payload includes `agent_id`, `agent_type`, `session_id`, **plus** `agent_transcript_path` and `last_assistant_message`. |
| P3 | `SubagentStop` `additionalContext` reaches parent | **FAIL** (2 rounds) | Fallback engaged: SubagentStop hooks are disk-only; briefing goes through `<mission>/hints/orchestrator-hint.txt`, read at the top of every orchestrator turn. |
| P4 | `SessionStart` `compact`/`resume` matchers | **PASS** (`resume` verified; `compact` same mechanics, verify on first interactive use) | H-G uses `additionalContext` (supported for SessionStart per docs). |
| P5 | Plugin skill invocable as `/probe:probe5` | **PASS** | Skills ship as `skills/<name>/SKILL.md`. |
| P6 | `tools: Agent(plugin:agent)` allowlist constrains spawns | **PASS** | Non-allowed agent spawn errors with "Agent type not found". |
| P7 | Parallel `Agent(...)` calls in one turn | **PASS** | Scrutiny fan-out is safe (3 children verified). |
| P8 | Headless `AskUserQuestion` via stream-json | **FAIL** (denied in `--print`; model hallucinates an answer) | Fallback engaged: `MISSIONS_HEADLESS=1` makes the orchestrator read `planning/interview-answers.json` instead of calling AskUserQuestion. |
| P9 | `claude plugin validate` + local marketplace install | **PASS** | Re-validated locally on this machine (see below). |

## Critical sub-finding (P12 follow-up to P2/P3)

The same `SubagentStop` fires up to **~10 times** for one subagent stop if the hook does
work. **Every** SubagentStop hook must begin with the short-circuit:

```bash
ACTIVE="$(printf '%s' "$INPUT" | jq -r '.stop_hook_active // false')"
if [ "$ACTIVE" = "true" ]; then exit 0; fi
```

## Local re-verification log

- P9: `claude plugin validate plugins/missions` — run on 2026-06-10 against this repo (see CI/test output).
