# Session Bootstrap — Issue #130

Execute Issue #130 `INFRA-005-TARGET-RUNNER-CONTROL-PLANE-RECOVERY` as an `env:ubuntu-arm64` infrastructure Worker.

1. Read live #130, `task.md`, `AGENTS.md`, `docs/tasks/issue-lifecycle-protocol.md`, #21 Final Acceptance/correction, #87 Final Acceptance, and the trigger deployment run/job identity.
2. Claim only from OPEN + `status:ready + env:ubuntu-arm64 + no owner`; determine Attempt 1 and read back the claim.
3. Diagnose only with bounded non-secret runner/control/process/_diag evidence allowed by `task.md`.
4. Do not re-register, rotate credentials, inspect Secret files, install packages, use sudo/ADB, modify product code, dispatch a second R002 workflow, or run Bilibili/#67/#68/R003 work.
5. Perform at most one evidence-directed bounded recovery cycle of the existing `gateway-runnerctl` control plane.
6. If Listener recovers, prove uid 999 boundary and observe whether already-queued job `99273465231` naturally leaves queued and exposes normal steps/logs.
7. If recovery still fails, report BLOCKED with exact classified evidence and minimal resume condition; do not loop restarts.
8. Before every terminal Issue mutation, use the fresh terminal-write authority guard. Report → status transition → owner release → STOP.
