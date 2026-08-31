# Session Bootstrap — Issue #142

Execute Issue #142 `INFRA-PHONE-MGMT-READINESS-LIVE` as the cloud readiness Worker.

1. Read live #142, this Task Contract, #139 Final Acceptance, `docs/tasks/139-infra-phone-mgmt-reliability-prep/runbook.md`, `scripts/phone_mgmt_readiness.py`, `AGENTS.md`, `.agents/skills/task-worker/SKILL.md`, and lifecycle protocols before claim.
2. Claim only from OPEN + `status:ready + env:cloud + no owner`.
3. Run exactly one layered observation set: Tailnet -> SSH TCP/auth -> Ubuntu -> persistent context. Stop at the first failed layer; never retry or start a second set.
4. Attempt 1 is verification-only. Do not start/stop/configure Tailscale, sshd, wake-lock, chroot, tmux, Runner, workflow, browser, Bilibili or product state.
5. Suppress raw network/auth output. Durable evidence contains only normalized booleans/unknowns and the accepted classifier result.
6. ControlMaster/control socket may be observed only as non-authoritative context and must not affect classifier authority.
7. Run the accepted `scripts/phone_mgmt_readiness.py` offline on the normalized snapshot.
8. PASS only on `UBUNTU_PERSISTENT_READY / claim_allowed=true / reason=authorized`.
9. Any lower/ambiguous state is BLOCKED; report the failed layer and STOP without downstream #113/#131 claim.
10. Before every terminal Issue mutation, use the current fresh terminal-write authority guard; then transition to `status:review|status:blocked`, release owner and STOP.