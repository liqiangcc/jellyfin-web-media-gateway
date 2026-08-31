# Session Bootstrap — Issue #144

Execute Issue #144 `INFRA-PHONE-MGMT-READINESS-REFRESH` as a cloud verification Worker.

1. Read live #144, this Task Contract, #139 Final Acceptance, accepted `scripts/phone_mgmt_readiness.py`, `docs/tasks/139-infra-phone-mgmt-reliability-prep/runbook.md`, `AGENTS.md`, `.agents/skills/task-worker/SKILL.md`, lifecycle protocols, and current #113/#131 authority before claim.
2. Claim only from OPEN + `status:ready + env:cloud + no owner`; read back claim.
3. Execute exactly one layered readiness set: Tailnet -> SSH TCP/auth -> Ubuntu -> persistence. Failure/unknown at an earlier layer suppresses deeper probes.
4. Use bounded non-interactive checks only; no password prompt, auth guessing, retry loop, ControlMaster requirement, raw host/IP/user/key/fingerprint/route/latency output.
5. Perform no phone/Tailscale/sshd/wake/chroot configuration or restart and no Runner/workflow/product/browser/Bilibili action.
6. Normalize only the five accepted booleans and run `scripts/phone_mgmt_readiness.py` offline.
7. PASS only on `UBUNTU_PERSISTENT_READY / claim_allowed=true / reason=authorized`; otherwise BLOCKED. Never claim #113/#131 from this Worker.
8. Before every terminal Issue mutation use the fresh terminal-write authority guard.
9. Report bounded normalized evidence only; release owner and STOP.