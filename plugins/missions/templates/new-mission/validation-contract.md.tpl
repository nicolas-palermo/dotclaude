# Validation contract — {{OBJECTIVE_TITLE}}

Every assertion has a stable VAL-ID (`VAL-<DOMAIN>-NNN`). Workers reference them in
`tests.added[].cases[].verifies`; the handoff hook rejects unknown VAL-IDs. The
orchestrator tracks status in `validation-state.json`.

<!-- One section per domain. Format per assertion:
## VAL-<DOMAIN>

- **VAL-<DOMAIN>-001** — <falsifiable assertion>. Tool: <cargo test|pytest|vitest|agent-browser|HITL>. Evidence: <what proves it>.
-->
